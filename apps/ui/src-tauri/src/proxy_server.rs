use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use tokio::io::{self, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use url::Url;

use crate::config::{Config, ProxyProtocol};
use crate::credentials::{build_auth_header, Credentials};
use crate::net::{with_timeout, CLIENT_REQUEST_TIMEOUT, CONNECT_TIMEOUT, HANDSHAKE_TIMEOUT};
use crate::socks::connect_via_socks5;

/// Defensive limits to stop a local client from exhausting memory by sending an
/// unbounded request line or header block. Bodies are streamed, never buffered,
/// so they need no size cap.
const MAX_LINE_BYTES: u64 = 16 * 1024; // 16 KiB per request/header line
const MAX_HEADERS: usize = 200;

/// Upper bound on client connections serviced at once. Past this, new
/// connections wait for a free slot rather than spawning tasks (and upstream
/// sockets) without limit. Loopback + same-user means this is a belt-and-braces
/// resource guard, not a hard security boundary.
const MAX_CONCURRENT_CONNECTIONS: usize = 512;

type ClientReader = BufReader<tokio::net::tcp::OwnedReadHalf>;

/// Read one line, never buffering more than `MAX_LINE_BYTES`. Returns bytes read
/// (0 on EOF); errors if the cap is hit before a newline.
async fn read_line_limited(reader: &mut ClientReader, line: &mut String) -> Result<usize> {
    let n = AsyncReadExt::take(&mut *reader, MAX_LINE_BYTES)
        .read_line(line)
        .await?;
    if n as u64 == MAX_LINE_BYTES && !line.ends_with('\n') {
        bail!("Request line exceeded {} bytes", MAX_LINE_BYTES);
    }
    Ok(n)
}

/// A valid HTTP header field-name is an RFC 7230 token. Rejecting non-token names
/// stops a client from hiding a separator inside the name to smuggle a second
/// header when we re-emit the request to the upstream proxy.
fn is_valid_header_name(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

/// A header field-value may legally contain visible ASCII, spaces, tabs, and
/// high bytes (obs-text), but never other control characters. In particular an
/// interior bare CR survives `.trim()`, and if forwarded verbatim it can split
/// into a second header line at a lenient upstream parser (request smuggling), so
/// reject any control character except horizontal tab.
fn is_valid_header_value(value: &str) -> bool {
    value.chars().all(|c| c == '\t' || !c.is_ascii_control())
}

/// Serve the proxy on an already-bound listener. Binding is the caller's job so
/// that a port conflict fails `ServerManager::start` loudly, rather than dying
/// inside a spawned task while the app reports the server as running.
pub async fn run_proxy_server(
    listener: TcpListener,
    config: Config,
    credentials: Credentials,
) -> Result<()> {
    println!("Proxy wrapper running on http://{}", listener.local_addr()?);

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));

    let config = Arc::new(config);
    let credentials = Arc::new(credentials);
    let auth_header = Arc::new(build_auth_header(&credentials));

    // Connection tasks live in a JoinSet rather than being detached, so that
    // when this future is dropped (ServerManager::stop aborts it) every
    // in-flight connection and tunnel is aborted with it, instead of surviving
    // until the peer closes.
    let mut connections = JoinSet::new();

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (socket, addr) = accepted?;

                // Take a slot before spawning. When all slots are in use this
                // awaits, applying back-pressure (new connections queue in the
                // OS backlog) instead of growing tasks and upstream sockets
                // without bound.
                let permit = Arc::clone(&semaphore).acquire_owned().await?;

                let config = Arc::clone(&config);
                let credentials = Arc::clone(&credentials);
                let auth_header = Arc::clone(&auth_header);

                connections.spawn(async move {
                    let _permit = permit; // held for the connection's lifetime
                    if let Err(err) = handle_client(socket, config, credentials, auth_header).await {
                        eprintln!("Client {} error: {:?}", addr, err);
                    }
                });
            }
            // Reap finished tasks; a JoinSet retains results until joined.
            Some(_) = connections.join_next() => {}
        }
    }
}

async fn handle_client(
    socket: TcpStream,
    config: Arc<Config>,
    credentials: Arc<Credentials>,
    auth_header: Arc<String>,
) -> Result<()> {
    let (client_read, client_write) = socket.into_split();
    let mut reader = BufReader::new(client_read);

    // Bound the whole request-head read so a client that connects and stalls
    // (slow-loris) cannot hold a task open indefinitely.
    let parsed = with_timeout(CLIENT_REQUEST_TIMEOUT, "client request read", async {
        let mut request_line = String::new();
        if read_line_limited(&mut reader, &mut request_line).await? == 0 {
            return Ok(None);
        }

        let line = request_line.trim_end_matches(&['\r', '\n'][..]);
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 3 {
            // Don't echo the raw line: for a forward-proxy request the target is
            // a full URL whose query string can carry secrets, and this error is
            // printed to stderr.
            bail!("Malformed request line from client (expected method, target, and version)");
        }

        let method = parts[0].to_string();
        let target = parts[1].to_string();
        let version = parts[2].to_string();

        let mut headers = Vec::new();
        loop {
            let mut line = String::new();
            let bytes = read_line_limited(&mut reader, &mut line).await?;
            if bytes == 0 {
                break;
            }

            if line == "\r\n" || line.trim().is_empty() {
                break;
            }

            if headers.len() >= MAX_HEADERS {
                bail!("Request exceeded {} headers", MAX_HEADERS);
            }

            if let Some((name, value)) = line.split_once(':') {
                let name = name.trim();
                let value = value.trim();
                if !is_valid_header_name(name) {
                    bail!("Request header has an invalid name");
                }
                if !is_valid_header_value(value) {
                    bail!("Request header value contains control characters");
                }
                headers.push((name.to_string(), value.to_string()));
            }
        }

        Ok(Some((method, target, version, headers)))
    })
    .await?;

    let Some((method, target, version, headers)) = parsed else {
        return Ok(());
    };

    if method.eq_ignore_ascii_case("CONNECT") {
        // Keep the BufReader: bytes the client pipelined after the request head
        // (e.g. optimistic TLS ClientHello) sit in its buffer and would be
        // silently dropped by `into_inner()`.
        handle_connect(
            target,
            version,
            reader,
            client_write,
            &config,
            &credentials,
            &auth_header,
        )
        .await?;
    } else {
        forward_http(
            method,
            target,
            version,
            headers,
            reader,
            client_write,
            &config,
            &credentials,
            &auth_header,
        )
        .await?;
    }

    Ok(())
}

async fn handle_connect(
    target: String,
    version: String,
    client_reader: ClientReader,
    client_writer: tokio::net::tcp::OwnedWriteHalf,
    config: &Config,
    credentials: &Credentials,
    auth_header: &str,
) -> Result<()> {
    match config.proxy_protocol {
        ProxyProtocol::Http => {
            handle_http_connect(
                target,
                version,
                client_reader,
                client_writer,
                config,
                auth_header,
            )
            .await
        }
        ProxyProtocol::Socks5 => {
            handle_socks_connect(
                target,
                version,
                client_reader,
                client_writer,
                config,
                credentials,
            )
            .await
        }
    }
}

/// Extract the status code from a proxy's CONNECT response status line, e.g.
/// `b"HTTP/1.1 200 Connection established\r\n"` -> `Some(200)`. Returns `None`
/// for an empty or unparseable line.
pub(crate) fn connect_status_code(response_line: &[u8]) -> Option<u16> {
    String::from_utf8_lossy(response_line)
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
}

async fn handle_http_connect(
    target: String,
    version: String,
    client_reader: ClientReader,
    mut client_writer: tokio::net::tcp::OwnedWriteHalf,
    config: &Config,
    auth_header: &str,
) -> Result<()> {
    let mut proxy_stream = with_timeout(CONNECT_TIMEOUT, "proxy connect", async {
        Ok(TcpStream::connect((&config.proxy_host[..], config.proxy_port)).await?)
    })
    .await?;

    let response_line = with_timeout(HANDSHAKE_TIMEOUT, "proxy CONNECT handshake", async {
        let connect_request = format!(
            "CONNECT {} {}\r\nProxy-Authorization: {}\r\n\r\n",
            target, version, auth_header
        );
        proxy_stream.write_all(connect_request.as_bytes()).await?;
        proxy_stream.flush().await?;

        let mut response_line = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            let n = proxy_stream.read(&mut byte).await?;
            if n == 0 {
                break;
            }
            response_line.push(byte[0]);
            if byte[0] == b'\n' {
                break;
            }
            if response_line.len() as u64 >= MAX_LINE_BYTES {
                bail!(
                    "Proxy CONNECT response line exceeded {} bytes",
                    MAX_LINE_BYTES
                );
            }
        }
        Ok(response_line)
    })
    .await?;

    if response_line.is_empty() {
        bail!("Upstream proxy closed the connection without responding to CONNECT");
    }

    // Relay the upstream's status line to the client either way, so a rejected
    // tunnel surfaces the real reason (e.g. 407) instead of hanging silently.
    client_writer.write_all(&response_line).await?;
    client_writer.flush().await?;

    // Only tunnel on a 2xx CONNECT response. On any other status (407, 403,
    // 502, …) there is no tunnel, and piping would just shovel the upstream's
    // error body at the client as though it were tunnelled data.
    if !matches!(connect_status_code(&response_line), Some(200..=299)) {
        bail!(
            "Upstream proxy rejected CONNECT to {}: {}",
            target,
            String::from_utf8_lossy(&response_line).trim()
        );
    }

    pipe_streams(client_reader, client_writer, proxy_stream).await
}

async fn handle_socks_connect(
    target: String,
    version: String,
    client_reader: ClientReader,
    mut client_writer: tokio::net::tcp::OwnedWriteHalf,
    config: &Config,
    credentials: &Credentials,
) -> Result<()> {
    let (host, port) = parse_host_and_port(&target)?;
    let proxy_stream = connect_via_socks5(
        &config.proxy_host,
        config.proxy_port,
        credentials,
        &host,
        port,
    )
    .await?;

    let response = format!("{version} 200 Connection Established\r\n\r\n");
    client_writer.write_all(response.as_bytes()).await?;
    client_writer.flush().await?;

    pipe_streams(client_reader, client_writer, proxy_stream).await
}

/// Hop-by-hop headers are connection-scoped and must not be forwarded. The
/// client's `Proxy-Authorization` is replaced with our own upstream credentials.
fn is_hop_by_hop_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("connection")
        || name.eq_ignore_ascii_case("proxy-connection")
        || name.eq_ignore_ascii_case("keep-alive")
        || name.eq_ignore_ascii_case("proxy-authorization")
}

/// Serialize the request head for the upstream. `proxy_auth` is set when talking
/// to an upstream HTTP proxy; `fallback_host` supplies a `Host` header for
/// origin-form requests when the client did not send one.
///
/// `Connection: close` is always forced: connection reuse is deliberately
/// unsupported, and the close guarantees the upstream ends the connection after
/// one response, which is what terminates the relay in `forward_http`.
fn build_forward_head(
    method: &str,
    target: &str,
    version: &str,
    headers: &[(String, String)],
    proxy_auth: Option<&str>,
    fallback_host: Option<&str>,
) -> String {
    let mut head = format!("{method} {target} {version}\r\n");
    if let Some(auth) = proxy_auth {
        head.push_str(&format!("Proxy-Authorization: {auth}\r\n"));
    }
    head.push_str("Connection: close\r\n");

    let mut has_host_header = false;
    for (name, value) in headers {
        if is_hop_by_hop_header(name) {
            continue;
        }
        if name.eq_ignore_ascii_case("host") {
            has_host_header = true;
        }
        head.push_str(&format!("{name}: {value}\r\n"));
    }

    if !has_host_header {
        if let Some(host) = fallback_host {
            head.push_str(&format!("Host: {host}\r\n"));
        }
    }

    head.push_str("\r\n");
    head
}

#[allow(clippy::too_many_arguments)]
async fn forward_http(
    method: String,
    target: String,
    version: String,
    headers: Vec<(String, String)>,
    reader: ClientReader,
    client_writer: tokio::net::tcp::OwnedWriteHalf,
    config: &Config,
    credentials: &Credentials,
    auth_header: &str,
) -> Result<()> {
    let (mut proxy_stream, request_head) = match config.proxy_protocol {
        ProxyProtocol::Http => {
            let stream = with_timeout(CONNECT_TIMEOUT, "proxy connect", async {
                Ok(TcpStream::connect((&config.proxy_host[..], config.proxy_port)).await?)
            })
            .await?;

            // The upstream proxy expects the absolute-form target as sent.
            let head = build_forward_head(
                &method,
                &target,
                &version,
                &headers,
                Some(auth_header),
                None,
            );
            (stream, head)
        }
        ProxyProtocol::Socks5 => {
            let parsed = Url::parse(&target)
                .or_else(|_| Url::parse(&format!("http://{}", target)))
                .context("HTTP request target is not a valid URL")?;

            let host = parsed
                .host_str()
                .ok_or_else(|| anyhow!("Request URL is missing host"))?
                .to_string();
            let port = parsed
                .port_or_known_default()
                .ok_or_else(|| anyhow!("Request URL is missing port"))?;

            let stream = connect_via_socks5(
                &config.proxy_host,
                config.proxy_port,
                credentials,
                &host,
                port,
            )
            .await?;

            // Talking to the origin server through the tunnel, so rewrite the
            // absolute-form target to origin-form and make sure a Host header
            // is present.
            let mut request_target = parsed.path().to_string();
            if request_target.is_empty() {
                request_target.push('/');
            }
            if let Some(query) = parsed.query() {
                request_target.push('?');
                request_target.push_str(query);
            }

            let host_header = if (port == 80 && parsed.scheme() == "http")
                || (port == 443 && parsed.scheme() == "https")
            {
                host.clone()
            } else {
                format!("{}:{}", host, port)
            };

            let head = build_forward_head(
                &method,
                &request_target,
                &version,
                &headers,
                None,
                Some(&host_header),
            );
            (stream, head)
        }
    };

    proxy_stream.write_all(request_head.as_bytes()).await?;
    proxy_stream.flush().await?;

    // Stream the body (if any) and the response concurrently instead of
    // buffering the body until client EOF — keep-alive clients never send EOF
    // (they hold the connection open while waiting for the response), so
    // buffering would deadlock every plain-HTTP request. The upstream owns body
    // framing (Content-Length or chunked), and the forced `Connection: close`
    // guarantees the relay terminates once the response is complete.
    relay_forwarded_request(reader, client_writer, proxy_stream).await
}

/// Relay a forwarded request's remaining bytes and its response. Unlike a
/// CONNECT tunnel, only the upstream's close ends the exchange: a client may
/// legitimately half-close after sending its request (EOF marks the end of the
/// body), and the response must still be relayed back to it.
async fn relay_forwarded_request(
    mut client_reader: ClientReader,
    mut client_writer: tokio::net::tcp::OwnedWriteHalf,
    proxy_stream: TcpStream,
) -> Result<()> {
    let (mut proxy_read, mut proxy_write) = proxy_stream.into_split();

    let feed_request = async {
        let _ = io::copy(&mut client_reader, &mut proxy_write).await;
        // Propagate the client's half-close so an upstream reading an
        // EOF-terminated body sees the end of the request.
        let _ = proxy_write.shutdown().await;
        std::future::pending::<()>().await
    };

    tokio::select! {
        _ = feed_request => unreachable!("feed_request never completes"),
        result = io::copy(&mut proxy_read, &mut client_writer) => {
            result?;
        }
    }

    Ok(())
}

async fn pipe_streams<R>(
    mut client_reader: R,
    mut client_writer: tokio::net::tcp::OwnedWriteHalf,
    proxy_stream: TcpStream,
) -> Result<()>
where
    R: AsyncRead + Unpin,
{
    let (mut proxy_read, mut proxy_write) = proxy_stream.into_split();
    let client_to_proxy = io::copy(&mut client_reader, &mut proxy_write);
    let proxy_to_client = io::copy(&mut proxy_read, &mut client_writer);
    tokio::select! {
        _ = client_to_proxy => {},
        _ = proxy_to_client => {},
    }
    Ok(())
}

fn parse_host_and_port(target: &str) -> Result<(String, u16)> {
    let parsed = Url::parse(&format!("http://{}", target))
        .context("CONNECT target is not a valid host:port pair")?;

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("CONNECT target is missing host"))?;
    let port = parsed
        .port()
        .ok_or_else(|| anyhow!("CONNECT target is missing port"))?;

    Ok((host.to_string(), port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_header_names_accepted() {
        assert!(is_valid_header_name("Host"));
        assert!(is_valid_header_name("Content-Type"));
        assert!(is_valid_header_name("X-Custom_Header"));
        assert!(is_valid_header_name("X-Forwarded-For"));
    }

    #[test]
    fn invalid_header_names_rejected() {
        assert!(!is_valid_header_name("")); // empty
        assert!(!is_valid_header_name("Has Space")); // SP is not a tchar
        assert!(!is_valid_header_name("Bad\rName")); // bare CR
        assert!(!is_valid_header_name("Colon:In:Name")); // ':' is not a tchar
    }

    #[test]
    fn valid_header_values_accepted() {
        assert!(is_valid_header_value("text/html"));
        assert!(is_valid_header_value("Mozilla/5.0 (compatible)"));
        assert!(is_valid_header_value("value\twith\ttabs")); // HTAB is allowed
        assert!(is_valid_header_value("high-byte-\u{00ff}")); // obs-text allowed
        assert!(is_valid_header_value("")); // an empty value is legal
    }

    #[test]
    fn header_value_with_control_chars_rejected() {
        // The smuggling primitive: an interior CR that survives trimming and
        // would split into a second header line at a lenient upstream parser.
        assert!(!is_valid_header_value("bar\rHost: evil"));
        assert!(!is_valid_header_value("bar\r\nHost: evil"));
        assert!(!is_valid_header_value("bar\nHost: evil"));
        assert!(!is_valid_header_value("null\0byte"));
    }

    #[test]
    fn connect_status_code_parses_success() {
        assert_eq!(
            connect_status_code(b"HTTP/1.1 200 Connection established\r\n"),
            Some(200)
        );
        assert_eq!(
            connect_status_code(b"HTTP/1.0 204 No Content\r\n"),
            Some(204)
        );
    }

    #[test]
    fn connect_status_code_parses_failure() {
        assert_eq!(
            connect_status_code(b"HTTP/1.1 407 Proxy Authentication Required\r\n"),
            Some(407)
        );
        assert_eq!(
            connect_status_code(b"HTTP/1.1 502 Bad Gateway\r\n"),
            Some(502)
        );
    }

    #[test]
    fn connect_status_code_handles_garbage() {
        assert_eq!(connect_status_code(b""), None);
        assert_eq!(connect_status_code(b"garbage\r\n"), None);
        assert_eq!(connect_status_code(b"HTTP/1.1\r\n"), None);
    }

    #[test]
    fn forward_head_strips_hop_by_hop_and_forces_close() {
        let headers = vec![
            ("Host".to_string(), "example.com".to_string()),
            ("Connection".to_string(), "keep-alive".to_string()),
            ("Keep-Alive".to_string(), "timeout=5".to_string()),
            ("Proxy-Connection".to_string(), "keep-alive".to_string()),
            (
                "Proxy-Authorization".to_string(),
                "Basic c3RvbGVu".to_string(),
            ),
            ("Accept".to_string(), "*/*".to_string()),
        ];
        let head = build_forward_head(
            "GET",
            "http://example.com/",
            "HTTP/1.1",
            &headers,
            Some("Basic dXNlcjpwYXNz"),
            None,
        );

        assert!(head.starts_with("GET http://example.com/ HTTP/1.1\r\n"));
        assert!(head.contains("Connection: close\r\n"));
        assert!(!head.to_lowercase().contains("keep-alive"));
        // The client's own Proxy-Authorization must be replaced, not forwarded.
        assert!(!head.contains("c3RvbGVu"));
        assert!(head.contains("Proxy-Authorization: Basic dXNlcjpwYXNz\r\n"));
        assert!(head.contains("Accept: */*\r\n"));
        assert!(head.ends_with("\r\n\r\n"));
    }

    #[test]
    fn forward_head_adds_fallback_host_only_when_missing() {
        let headers = vec![("Accept".to_string(), "*/*".to_string())];
        let head = build_forward_head(
            "GET",
            "/",
            "HTTP/1.1",
            &headers,
            None,
            Some("example.com:8080"),
        );
        assert!(head.contains("Host: example.com:8080\r\n"));

        let headers = vec![("Host".to_string(), "client-host".to_string())];
        let head = build_forward_head("GET", "/", "HTTP/1.1", &headers, None, Some("fallback"));
        assert!(head.contains("Host: client-host\r\n"));
        assert!(!head.contains("fallback"));
    }

    /// End-to-end regression test for the keep-alive stall: a client that sends
    /// a plain-HTTP request and keeps its write half open (as every HTTP/1.1
    /// keep-alive client does) must still receive the response promptly. The old
    /// implementation buffered the body until client EOF, deadlocking here for
    /// 60 seconds and then dropping the connection.
    #[tokio::test]
    async fn keep_alive_client_gets_plain_http_response_without_stalling() {
        use std::time::Duration;

        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_port = upstream.local_addr().unwrap().port();

        // Mock upstream HTTP proxy: read one request head, send a response,
        // close (honouring Connection: close), and hand the head back for
        // inspection.
        let upstream_task = tokio::spawn(async move {
            let (mut sock, _) = upstream.accept().await.unwrap();
            let mut head = Vec::new();
            let mut byte = [0u8; 1];
            while !head.ends_with(b"\r\n\r\n") {
                sock.read_exact(&mut byte).await.unwrap();
                head.push(byte[0]);
            }
            sock.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello",
            )
            .await
            .unwrap();
            sock.shutdown().await.unwrap();
            String::from_utf8(head).unwrap()
        });

        let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = proxy_listener.local_addr().unwrap().port();

        let config = Config {
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: upstream_port,
            proxy_protocol: ProxyProtocol::Http,
            local_proxy_port: local_port,
            pac_port: 0,
            bypass_domains: vec![],
            username: "user".to_string(),
            password: "pass".to_string(),
            base_dir: std::path::PathBuf::from("/tmp"),
        };
        let credentials = Credentials {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        tokio::spawn(run_proxy_server(proxy_listener, config, credentials));

        let mut client = TcpStream::connect(("127.0.0.1", local_port)).await.unwrap();

        client
            .write_all(
                b"GET http://example.test/ HTTP/1.1\r\nHost: example.test\r\nConnection: keep-alive\r\n\r\n",
            )
            .await
            .unwrap();

        // Deliberately keep the write half open and just wait for the response.
        let mut response = Vec::new();
        tokio::time::timeout(Duration::from_secs(5), client.read_to_end(&mut response))
            .await
            .expect("proxy stalled instead of relaying the response")
            .unwrap();

        let response = String::from_utf8_lossy(&response);
        assert!(
            response.contains("hello"),
            "unexpected response: {response}"
        );

        let upstream_head = upstream_task.await.unwrap();
        assert!(upstream_head.to_lowercase().contains("connection: close"));
        assert!(!upstream_head.to_lowercase().contains("keep-alive"));
        assert!(upstream_head.contains("Proxy-Authorization: Basic dXNlcjpwYXNz"));
    }

    /// A client that half-closes after sending its request (EOF-terminated
    /// body, e.g. `printf … | nc`) must still receive the response: the relay
    /// may only end when the upstream closes, not when the client's request
    /// side finishes.
    #[tokio::test]
    async fn half_closing_client_still_gets_response() {
        use std::time::Duration;

        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_port = upstream.local_addr().unwrap().port();

        let upstream_task = tokio::spawn(async move {
            let (mut sock, _) = upstream.accept().await.unwrap();
            let mut head = Vec::new();
            let mut byte = [0u8; 1];
            while !head.ends_with(b"\r\n\r\n") {
                sock.read_exact(&mut byte).await.unwrap();
                head.push(byte[0]);
            }
            sock.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello",
            )
            .await
            .unwrap();
            sock.shutdown().await.unwrap();
        });

        let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = proxy_listener.local_addr().unwrap().port();

        let config = Config {
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: upstream_port,
            proxy_protocol: ProxyProtocol::Http,
            local_proxy_port: local_port,
            pac_port: 0,
            bypass_domains: vec![],
            username: "user".to_string(),
            password: "pass".to_string(),
            base_dir: std::path::PathBuf::from("/tmp"),
        };
        let credentials = Credentials {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        tokio::spawn(run_proxy_server(proxy_listener, config, credentials));

        let mut client = TcpStream::connect(("127.0.0.1", local_port)).await.unwrap();
        client
            .write_all(b"GET http://example.test/ HTTP/1.1\r\nHost: example.test\r\n\r\n")
            .await
            .unwrap();
        client.shutdown().await.unwrap();

        let mut response = Vec::new();
        tokio::time::timeout(Duration::from_secs(5), client.read_to_end(&mut response))
            .await
            .expect("response was dropped for a half-closed client")
            .unwrap();
        assert!(String::from_utf8_lossy(&response).contains("hello"));

        upstream_task.await.unwrap();
    }

    /// Aborting the server task must tear down established tunnels, not just
    /// the accept loop. Previously connection tasks were detached, so an open
    /// CONNECT tunnel kept relaying after the user hit "Disable".
    #[tokio::test]
    async fn aborting_server_closes_established_tunnels() {
        use std::time::Duration;

        // Mock upstream proxy: accept the CONNECT, reply 200, then hold the
        // tunnel open indefinitely.
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_port = upstream.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut sock, _) = upstream.accept().await.unwrap();
            let mut head = Vec::new();
            let mut byte = [0u8; 1];
            while !head.ends_with(b"\r\n\r\n") {
                sock.read_exact(&mut byte).await.unwrap();
                head.push(byte[0]);
            }
            sock.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
            let mut buf = [0u8; 1024];
            while sock.read(&mut buf).await.unwrap_or(0) > 0 {}
        });

        let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = proxy_listener.local_addr().unwrap().port();

        let config = Config {
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: upstream_port,
            proxy_protocol: ProxyProtocol::Http,
            local_proxy_port: local_port,
            pac_port: 0,
            bypass_domains: vec![],
            username: "user".to_string(),
            password: "pass".to_string(),
            base_dir: std::path::PathBuf::from("/tmp"),
        };
        let credentials = Credentials {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        let server = tokio::spawn(run_proxy_server(proxy_listener, config, credentials));

        let mut client = TcpStream::connect(("127.0.0.1", local_port)).await.unwrap();
        client
            .write_all(b"CONNECT example.test:443 HTTP/1.1\r\n\r\n")
            .await
            .unwrap();

        // Read until the relayed 200 response (status line + blank line) is in.
        let mut response = Vec::new();
        let mut byte = [0u8; 1];
        while !response.ends_with(b"\r\n\r\n") {
            client.read_exact(&mut byte).await.unwrap();
            response.push(byte[0]);
        }
        assert!(String::from_utf8_lossy(&response).contains("200"));

        server.abort();

        // The tunnel must close promptly once the server is gone.
        let mut buf = [0u8; 16];
        match tokio::time::timeout(Duration::from_secs(5), client.read(&mut buf)).await {
            Ok(Ok(0)) | Ok(Err(_)) => {} // closed (EOF or reset)
            Ok(Ok(n)) => panic!("unexpected {n} bytes through a dead tunnel"),
            Err(_) => panic!("tunnel survived server abort"),
        }
    }
}
