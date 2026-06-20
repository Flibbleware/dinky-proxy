use std::future::Future;
use std::time::Duration;

use anyhow::{bail, Result};

/// Time allowed to establish a TCP connection to an upstream proxy.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Time allowed for a proxy handshake (HTTP CONNECT reply or SOCKS5 negotiation)
/// to complete once connected.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

/// Time allowed for a client to send its full request line and headers. Bounds
/// slow-loris style clients that connect and then stall without sending data.
pub const CLIENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Time allowed to read a client's request body once the head has been parsed.
/// Bounds a client that sends headers and then stalls part-way through (or never
/// closes) its body, so the connection cannot be held open indefinitely.
pub const CLIENT_BODY_TIMEOUT: Duration = Duration::from_secs(60);

/// Run `fut` with a deadline. On timeout the future is dropped (cancelling the
/// in-flight I/O) and a labelled error is returned instead of hanging forever.
pub async fn with_timeout<F, T>(duration: Duration, label: &str, fut: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    match tokio::time::timeout(duration, fut).await {
        Ok(result) => result,
        Err(_) => bail!("{label} timed out after {duration:?}"),
    }
}
