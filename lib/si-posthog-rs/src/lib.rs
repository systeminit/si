pub mod api;
pub mod client;
pub mod error;
pub mod sender;

use std::time::Duration;

pub use client::PosthogClient;
pub use error::PosthogResult;
pub use sender::PosthogSender;
use tokio::sync::mpsc;

pub fn new(
    api_endpoint: impl Into<String>,
    api_key: impl Into<String>,
    timeout: Duration,
) -> PosthogResult<(PosthogClient, PosthogSender)> {
    let (tx, rx) = mpsc::unbounded_channel();
    let client = PosthogClient::new(tx);
    let sender = PosthogSender::new(rx, api_endpoint.into(), api_key.into(), timeout)?;
    Ok((client, sender))
}
