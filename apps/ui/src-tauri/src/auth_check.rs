use anyhow::{bail, Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::config::{Config, ProxyProtocol};
use crate::credentials::{build_auth_header, Credentials};
use crate::net::{with_timeout, CONNECT_TIMEOUT, HANDSHAKE_TIMEOUT};
use crate::proxy_server::connect_status_code;
use crate::socks::connect_via_socks5;

pub async fn test_proxy_auth(config: &Config, credentials: &Credentials) -> Result<()> {
    println!("Checking proxy authentication...");

    match config.proxy_protocol {
        ProxyProtocol::Http => test_http_proxy(config, credentials).await,
        ProxyProtocol::Socks5 => test_socks_proxy(config, credentials).await,
    }
}

async fn test_http_proxy(config: &Config, credentials: &Credentials) -> Result<()> {
    let auth_header = build_auth_header(credentials);

    let stream = with_timeout(CONNECT_TIMEOUT, "proxy connect", async {
        TcpStream::connect((&config.proxy_host[..], config.proxy_port))
            .await
            .with_context(|| {
                format!(
                    "Could not connect to proxy {}:{}",
                    config.proxy_host, config.proxy_port
                )
            })
    })
    .await?;

    let test_host = "example.com:443";
    let request = format!(
        "CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Authorization: {}\r\n\r\n",
        test_host, test_host, auth_header
    );

    let (response_line, bytes) = with_timeout(HANDSHAKE_TIMEOUT, "proxy auth check", async {
        let mut stream = stream;
        stream.write_all(request.as_bytes()).await?;

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        let bytes = reader.read_line(&mut response_line).await?;
        Ok((response_line, bytes))
    })
    .await?;

    if bytes == 0 {
        bail!("No response received from proxy during authentication test");
    }

    println!("Proxy response: {}", response_line.trim());

    // Parse the status code out of the line rather than substring-matching:
    // "contains 200" would accept a 407 whose reason phrase happens to contain
    // "200" and reject nothing it shouldn't, hiding real auth failures.
    if matches!(
        connect_status_code(response_line.as_bytes()),
        Some(200..=299)
    ) {
        println!("Proxy authentication OK ✓\n");
        Ok(())
    } else {
        bail!("Proxy authentication failed: {}", response_line.trim());
    }
}

async fn test_socks_proxy(config: &Config, credentials: &Credentials) -> Result<()> {
    connect_via_socks5(
        &config.proxy_host,
        config.proxy_port,
        credentials,
        "example.com",
        443,
    )
    .await?;

    println!("SOCKS5 proxy authentication OK ✓\n");
    Ok(())
}
