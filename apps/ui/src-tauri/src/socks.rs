use anyhow::{bail, Context, Result};
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::credentials::Credentials;
use crate::net::{with_timeout, CONNECT_TIMEOUT, HANDSHAKE_TIMEOUT};

/// Establish a SOCKS5 connection to the target host using username/password auth.
pub async fn connect_via_socks5(
    proxy_host: &str,
    proxy_port: u16,
    credentials: &Credentials,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream> {
    ensure_credential_lengths(credentials)?;

    let mut stream = with_timeout(CONNECT_TIMEOUT, "SOCKS5 proxy connect", async {
        TcpStream::connect((proxy_host, proxy_port))
            .await
            .with_context(|| {
                format!(
                    "Could not connect to SOCKS5 proxy {}:{}",
                    proxy_host, proxy_port
                )
            })
    })
    .await?;

    with_timeout(HANDSHAKE_TIMEOUT, "SOCKS5 handshake", async {
        negotiate_with_password(&mut stream, credentials).await?;
        send_connect_request(&mut stream, target_host, target_port).await
    })
    .await?;

    Ok(stream)
}

fn ensure_credential_lengths(credentials: &Credentials) -> Result<()> {
    if credentials.username.len() > u8::MAX as usize {
        bail!("SOCKS5 username must not exceed 255 bytes");
    }

    if credentials.password.len() > u8::MAX as usize {
        bail!("SOCKS5 password must not exceed 255 bytes");
    }

    Ok(())
}

async fn negotiate_with_password(stream: &mut TcpStream, credentials: &Credentials) -> Result<()> {
    // Method selection: username/password only to avoid unauthenticated fallbacks.
    stream.write_all(&[0x05, 0x01, 0x02]).await?;

    let mut method_response = [0u8; 2];
    stream.read_exact(&mut method_response).await?;
    if method_response[1] != 0x02 {
        bail!(
            "SOCKS5 proxy does not allow username/password authentication (got method {:#04x})",
            method_response[1]
        );
    }

    let mut auth_request =
        Vec::with_capacity(3 + credentials.username.len() + credentials.password.len());
    auth_request.push(0x01); // auth version
    auth_request.push(credentials.username.len() as u8);
    auth_request.extend_from_slice(credentials.username.as_bytes());
    auth_request.push(credentials.password.len() as u8);
    auth_request.extend_from_slice(credentials.password.as_bytes());

    stream.write_all(&auth_request).await?;

    let mut auth_response = [0u8; 2];
    stream.read_exact(&mut auth_response).await?;
    if auth_response[1] != 0x00 {
        bail!(
            "SOCKS5 authentication failed with status {:#04x}",
            auth_response[1]
        );
    }

    Ok(())
}

async fn send_connect_request(
    stream: &mut TcpStream,
    target_host: &str,
    target_port: u16,
) -> Result<()> {
    let mut request = Vec::new();
    request.extend_from_slice(&[0x05, 0x01, 0x00]); // version, connect cmd, reserved

    match target_host.parse::<IpAddr>() {
        Ok(IpAddr::V4(addr)) => {
            request.push(0x01); // IPv4
            request.extend_from_slice(&addr.octets());
        }
        Ok(IpAddr::V6(addr)) => {
            request.push(0x04); // IPv6
            request.extend_from_slice(&addr.octets());
        }
        Err(_) => {
            let host_bytes = target_host.as_bytes();
            if host_bytes.len() > u8::MAX as usize {
                bail!("SOCKS5 target host is too long");
            }

            request.push(0x03); // domain
            request.push(host_bytes.len() as u8);
            request.extend_from_slice(host_bytes);
        }
    }

    request.extend_from_slice(&target_port.to_be_bytes());

    stream.write_all(&request).await?;

    // Response: ver, status, reserved, atyp, addr, port
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    if header[1] != 0x00 {
        bail!(
            "SOCKS5 connect command failed with status {:#04x}",
            header[1]
        );
    }

    match header[3] {
        0x01 => {
            let mut buf = [0u8; 6]; // IPv4 (4) + port (2)
            stream.read_exact(&mut buf).await?;
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut buf = vec![0u8; len[0] as usize + 2];
            stream.read_exact(&mut buf).await?;
        }
        0x04 => {
            let mut buf = [0u8; 18]; // IPv6 (16) + port (2)
            stream.read_exact(&mut buf).await?;
        }
        other => {
            bail!("Unexpected SOCKS5 address type in response: {:#04x}", other);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::Credentials;

    fn creds(username: &str, password: &str) -> Credentials {
        Credentials {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    #[test]
    fn valid_lengths_accepted() {
        assert!(ensure_credential_lengths(&creds("user", "pass")).is_ok());
    }

    #[test]
    fn long_username_rejected() {
        let long = "a".repeat(256);
        assert!(ensure_credential_lengths(&creds(&long, "pass")).is_err());
    }

    #[test]
    fn long_password_rejected() {
        let long = "a".repeat(256);
        assert!(ensure_credential_lengths(&creds("user", &long)).is_err());
    }
}
