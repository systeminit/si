use tokio::sync::mpsc;

mod api;
mod client;
mod config;
mod error;
mod sender;

pub use client::{FeatureFlag, PosthogClient};
pub use config::{PosthogConfig, PosthogConfigBuilder};
pub use error::{PosthogError, PosthogResult};
pub use sender::PosthogSender;

pub fn new() -> PosthogConfigBuilder {
    PosthogConfigBuilder::default()
}

pub fn from_config(config: &PosthogConfig) -> PosthogResult<(PosthogClient, PosthogSender)> {
    let (tx, rx) = mpsc::unbounded_channel();
    let client = PosthogClient::new(tx, config)?;
    let sender = PosthogSender::new(rx, config)?;
    Ok((client, sender))
}
