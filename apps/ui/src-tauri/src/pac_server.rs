use std::sync::Arc;

use anyhow::Result;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::net::{with_timeout, CLIENT_REQUEST_TIMEOUT};

/// Upper bound on concurrent PAC requests serviced at once. PAC fetches are
/// short-lived, so this only needs to absorb bursts (e.g. many tabs after a
/// network change) without spawning tasks without bound.
const MAX_CONCURRENT_PAC_CONNECTIONS: usize = 64;

/// Cap on the request head read before responding. The PAC handler ignores the
/// request, so this just stops a client from streaming an unbounded "line" with
/// no newline to exhaust memory.
const MAX_REQUEST_BYTES: u64 = 8 * 1024;

pub async fn run_pac_server(config: Config) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", config.pac_port)).await?;
    println!("PAC server running on http://localhost:{}", config.pac_port);

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_PAC_CONNECTIONS));

    loop {
        let (socket, addr) = listener.accept().await?;
        let permit = Arc::clone(&semaphore).acquire_owned().await?;
        let config = config.clone();

        tokio::spawn(async move {
            let _permit = permit; // held until the response is sent
            if let Err(err) = handle_pac_connection(socket, config).await {
                eprintln!("PAC client {} error: {:?}", addr, err);
            }
        });
    }
}

async fn handle_pac_connection(socket: TcpStream, config: Config) -> Result<()> {
    let (client_read, mut client_write) = socket.into_split();
    let mut reader = BufReader::new(client_read);

    // Drain the request head with a size cap and a timeout. The PAC response is
    // identical for every caller, so the request itself is ignored — but reading
    // it unbounded would let a client stall (slow-loris) or stream forever to
    // hold the task open and exhaust memory.
    let mut _request_line = String::new();
    with_timeout(CLIENT_REQUEST_TIMEOUT, "PAC request read", async {
        AsyncReadExt::take(&mut reader, MAX_REQUEST_BYTES)
            .read_line(&mut _request_line)
            .await?;
        Ok(())
    })
    .await?;

    let pac_file = config.pac_path();
    match fs::read_to_string(&pac_file).await {
        Ok(contents) => {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                contents.len(),
                contents
            );
            client_write.write_all(response.as_bytes()).await?;
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            client_write
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await?;
        }
        Err(err) => {
            client_write
                .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n")
                .await?;
            return Err(err.into());
        }
    }

    Ok(())
}

/// Generate the default PAC file from config.json so the served PAC stays in sync.
pub fn sync_default_pac(config: &Config) -> Result<()> {
    let pac_path = config.pac_path();
    if let Some(dir) = pac_path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let contents = build_default_pac(config);
    std::fs::write(&pac_path, contents)?;
    crate::config::restrict_to_owner(&pac_path)?;

    Ok(())
}

/// A bypass domain is safe to embed in the generated PAC JavaScript only if every
/// character is one that can legally appear in a hostname. This rejects quotes,
/// backslashes, and newlines that would otherwise break out of the JS string
/// literal (code injection) or produce invalid JS (which makes browsers fail open
/// to DIRECT for all traffic).
pub(crate) fn is_valid_bypass_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
}

pub(crate) fn build_default_pac(config: &Config) -> String {
    let mut rules = String::new();

    for domain in &config.bypass_domains {
        let trimmed = domain.trim();
        if !is_valid_bypass_domain(trimmed) {
            if !trimmed.is_empty() {
                eprintln!("[PAC] Skipping invalid bypass domain: {trimmed:?}");
            }
            continue;
        }

        rules.push_str(&format!(
            "    if (dnsDomainIs(host, \".{d}\") || host === \"{d}\") {{\n        return myProxy;\n    }}\n\n",
            d = trimmed
        ));
    }

    format!(
        "function FindProxyForURL(url, host) {{
    const myProxy = \"PROXY localhost:{port}\";

    if (host === \"localhost\" || host === \"127.0.0.1\") {{
        return \"DIRECT\";
    }}

{rules}    return \"DIRECT\";
}}",
        port = config.local_proxy_port,
        rules = rules
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ProxyProtocol};
    use std::path::PathBuf;

    fn config(bypass_domains: Vec<String>, local_proxy_port: u16) -> Config {
        Config {
            proxy_host: String::new(),
            proxy_port: 8080,
            proxy_protocol: ProxyProtocol::Http,
            local_proxy_port,
            pac_port: 8000,
            bypass_domains,
            username: String::new(),
            password: String::new(),
            base_dir: PathBuf::from("/tmp"),
        }
    }

    #[test]
    fn bypass_domains_appear_in_pac() {
        let pac = build_default_pac(&config(vec!["example.com".to_string()], 8888));
        assert!(pac.contains("dnsDomainIs(host, \".example.com\")"));
        assert!(pac.contains("host === \"example.com\""));
    }

    #[test]
    fn empty_bypass_list_produces_direct_only() {
        let pac = build_default_pac(&config(vec![], 8888));
        assert!(!pac.contains("dnsDomainIs"));
        assert!(pac.contains("return \"DIRECT\""));
    }

    #[test]
    fn whitespace_only_domains_are_skipped() {
        let pac = build_default_pac(&config(vec!["   ".to_string(), "".to_string()], 8888));
        assert!(!pac.contains("dnsDomainIs"));
    }

    #[test]
    fn correct_proxy_port_in_pac() {
        let pac = build_default_pac(&config(vec![], 9999));
        assert!(pac.contains("PROXY localhost:9999"));
    }

    #[test]
    fn valid_domains_accepted() {
        assert!(is_valid_bypass_domain("example.com"));
        assert!(is_valid_bypass_domain("sub-domain.example.co.uk"));
        assert!(is_valid_bypass_domain("internal"));
    }

    #[test]
    fn injection_chars_rejected() {
        // Quote/backslash/newline/space would break out of the JS string literal.
        assert!(!is_valid_bypass_domain("x\") || evil(url) || (\""));
        assert!(!is_valid_bypass_domain("a\"b"));
        assert!(!is_valid_bypass_domain("a\\b"));
        assert!(!is_valid_bypass_domain("a\nb"));
        assert!(!is_valid_bypass_domain("a b"));
        assert!(!is_valid_bypass_domain(""));
    }

    #[test]
    fn malicious_domain_is_not_emitted_into_pac() {
        let pac = build_default_pac(&config(
            vec![
                "evil\") || alert(1) || (\"".to_string(),
                "good.com".to_string(),
            ],
            8888,
        ));
        assert!(!pac.contains("alert(1)"));
        assert!(pac.contains("host === \"good.com\""));
    }
}
