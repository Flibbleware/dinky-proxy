use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use url::Url;

use crate::config::{Config, ProxyProtocol};
use crate::credentials::{build_auth_header, Credentials};
use crate::net::{
    with_timeout, CLIENT_BODY_TIMEOUT, CLIENT_REQUEST_TIMEOUT, CONNECT_TIMEOUT, HANDSHAKE_TIMEOUT,
};
use crate::socks::connect_via_socks5;

/// Defensive limits to stop a local client from exhausting memory by sending an
/// unbounded request line, header block, or body.
const MAX_LINE_BYTES: u64 = 16 * 1024; // 16 KiB per request/header line
const MAX_HEADERS: usize = 200;
const MAX_BODY_BYTES: u64 = 64 * 1024 * 1024; // 64 MiB request body cap

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

/// Read the request body, rejecting anything larger than `MAX_BODY_BYTES` and
/// bounding the total read time so a client cannot stall mid-body to pin the
/// connection (and its upstream socket) open.
async fn read_body_limited(reader: &mut ClientReader) -> Result<Vec<u8>> {
    with_timeout(CLIENT_BODY_TIMEOUT, "client body read", async {
        let mut body = Vec::new();
        AsyncReadExt::take(&mut *reader, MAX_BODY_BYTES + 1)
            .read_to_end(&mut body)
            .await?;
        if body.len() as u64 > MAX_BODY_BYTES {
            bail!("Request body exceeded {} bytes", MAX_BODY_BYTES);
        }
        Ok(body)
    })
    .await
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

pub async fn run_proxy_server(config: Config, credentials: Credentials) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", config.local_proxy_port)).await?;
    println!(
        "Proxy wrapper running on http://localhost:{}",
        config.local_proxy_port
    );

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));

    loop {
        let (socket, addr) = listener.accept().await?;

        // Take a slot before spawning. When all slots are in use this awaits,
        // applying back-pressure (new connections queue in the OS backlog)
        // instead of growing tasks and upstream sockets without bound.
        let permit = Arc::clone(&semaphore).acquire_owned().await?;

        let config = config.clone();
        let credentials = credentials.clone();
        let auth_header = build_auth_header(&credentials);

        tokio::spawn(async move {
            let _permit = permit; // held for the connection's lifetime
            if let Err(err) = handle_client(socket, config, credentials, auth_header).await {
                eprintln!("Client {} error: {:?}", addr, err);
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    config: Config,
    credentials: Credentials,
    auth_header: String,
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
        let client_reader = reader.into_inner();
        handle_connect(
            target,
            version,
            client_reader,
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
    client_reader: tokio::net::tcp::OwnedReadHalf,
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
fn connect_status_code(response_line: &[u8]) -> Option<u16> {
    String::from_utf8_lossy(response_line)
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
}

async fn handle_http_connect(
    target: String,
    version: String,
    client_reader: tokio::net::tcp::OwnedReadHalf,
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
    client_reader: tokio::net::tcp::OwnedReadHalf,
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

#[allow(clippy::too_many_arguments)]
async fn forward_http(
    method: String,
    target: String,
    version: String,
    headers: Vec<(String, String)>,
    reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    client_writer: tokio::net::tcp::OwnedWriteHalf,
    config: &Config,
    credentials: &Credentials,
    auth_header: &str,
) -> Result<()> {
    match config.proxy_protocol {
        ProxyProtocol::Http => {
            forward_http_via_http(
                method,
                target,
                version,
                headers,
                reader,
                client_writer,
                config,
                auth_header,
            )
            .await
        }
        ProxyProtocol::Socks5 => {
            forward_http_via_socks5(
                method,
                target,
                version,
                headers,
                reader,
                client_writer,
                config,
                credentials,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn forward_http_via_http(
    method: String,
    target: String,
    version: String,
    headers: Vec<(String, String)>,
    mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    mut client_writer: tokio::net::tcp::OwnedWriteHalf,
    config: &Config,
    auth_header: &str,
) -> Result<()> {
    let mut proxy_stream = with_timeout(CONNECT_TIMEOUT, "proxy connect", async {
        Ok(TcpStream::connect((&config.proxy_host[..], config.proxy_port)).await?)
    })
    .await?;

    let mut request = format!(
        "{} {} {}\r\nProxy-Authorization: {}\r\n",
        method, target, version, auth_header
    );

    for (name, value) in headers {
        if !name.eq_ignore_ascii_case("proxy-authorization") {
            request.push_str(&format!("{}: {}\r\n", name, value));
        }
    }

    request.push_str("\r\n");

    proxy_stream.write_all(request.as_bytes()).await?;

    let body = read_body_limited(&mut reader).await?;
    if !body.is_empty() {
        proxy_stream.write_all(&body).await?;
    }

    proxy_stream.flush().await?;
    io::copy(&mut proxy_stream, &mut client_writer).await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn forward_http_via_socks5(
    method: String,
    target: String,
    version: String,
    headers: Vec<(String, String)>,
    mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    mut client_writer: tokio::net::tcp::OwnedWriteHalf,
    config: &Config,
    credentials: &Credentials,
) -> Result<()> {
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

    let mut proxy_stream = connect_via_socks5(
        &config.proxy_host,
        config.proxy_port,
        credentials,
        &host,
        port,
    )
    .await?;

    let mut request_target = parsed.path().to_string();
    if request_target.is_empty() {
        request_target.push('/');
    }
    if let Some(query) = parsed.query() {
        request_target.push('?');
        request_target.push_str(query);
    }

    let mut request = format!("{method} {request_target} {version}\r\n");
    let mut has_host_header = false;

    for (name, value) in headers {
        if name.eq_ignore_ascii_case("proxy-authorization") {
            continue;
        }

        if name.eq_ignore_ascii_case("host") {
            has_host_header = true;
        }

        request.push_str(&format!("{}: {}\r\n", name, value));
    }

    if !has_host_header {
        let host_header = if (port == 80 && parsed.scheme() == "http")
            || (port == 443 && parsed.scheme() == "https")
        {
            host.clone()
        } else {
            format!("{}:{}", host, port)
        };
        request.push_str(&format!("Host: {}\r\n", host_header));
    }

    request.push_str("\r\n");

    proxy_stream.write_all(request.as_bytes()).await?;

    let body = read_body_limited(&mut reader).await?;
    if !body.is_empty() {
        proxy_stream.write_all(&body).await?;
    }

    proxy_stream.flush().await?;
    io::copy(&mut proxy_stream, &mut client_writer).await?;

    Ok(())
}

async fn pipe_streams(
    mut client_reader: tokio::net::tcp::OwnedReadHalf,
    mut client_writer: tokio::net::tcp::OwnedWriteHalf,
    proxy_stream: TcpStream,
) -> Result<()> {
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
}
