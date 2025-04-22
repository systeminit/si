use tokio::sync::mpsc;

mod api;
mod client;
mod config;
mod error;
mod sender;

pub use client::PosthogClient;
pub use config::{
    PosthogConfig,
    PosthogConfigBuilder,
};
pub use error::{
    PosthogError,
    PosthogResult,
};
pub use sender::PosthogSender;
use tokio_util::sync::CancellationToken;

pub fn new() -> PosthogConfigBuilder {
    PosthogConfigBuilder::default()
}

pub fn from_config(
    config: &PosthogConfig,
    token: CancellationToken,
) -> PosthogResult<(PosthogSender, PosthogClient)> {
    let (tx, rx) = mpsc::unbounded_channel();
    let sender = PosthogSender::new(rx, config, token)?;
    let client = PosthogClient::new(tx, config)?;
    Ok((sender, client))
}
