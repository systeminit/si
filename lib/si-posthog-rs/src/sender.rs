use telemetry::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

use crate::api::PosthogApiClient;
use crate::{PosthogConfig, api::PosthogMessage, error::PosthogResult};

#[derive(Debug)]
pub struct PosthogSender {
    rx: UnboundedReceiver<PosthogMessage>,
    api_client: PosthogApiClient,
    enabled: bool,
    token: CancellationToken,
}

impl PosthogSender {
    const NAME: &'static str = "si_posthog::posthog_sender";

    pub(crate) fn new(
        rx: UnboundedReceiver<PosthogMessage>,
        config: &PosthogConfig,
        token: CancellationToken,
    ) -> PosthogResult<PosthogSender> {
        let api_client = PosthogApiClient::new(config)?;

        Ok(PosthogSender {
            rx,
            api_client,
            enabled: config.enabled(),
            token,
        })
    }

    pub async fn run(mut self) {
        debug!(task = Self::NAME, "posthog sender running");
        if !self.enabled {
            debug!("posthog tracking is disabled");
        }

        loop {
            tokio::select! {
                _ = self.token.cancelled() => {
                    info!(task = Self::NAME, "received cancellation");
                    break;
                }
                maybe_msg = self.rx.recv() => {
                    match maybe_msg {
                        Some(msg) => self.process(msg).await,
                        None => break,
                    }
                }
            }
        }

        debug!(task = Self::NAME, "shutdown complete");
    }

    #[inline]
    async fn process(&mut self, msg: PosthogMessage) {
        trace!(task = Self::NAME, message = ?msg, "received message");
        if self.enabled {
            match msg {
                PosthogMessage::Event(event) => {
                    debug!(task = Self::NAME, event = ?event, "sending event to posthog");
                    if let Err(err) = self.api_client.send_event(event).await {
                        error!(
                            task = Self::NAME,
                            error = ?err,
                            "error sending event to posthog",
                        );
                    }
                }
                PosthogMessage::Disable => {
                    debug!(task = Self::NAME, "disabling posthog tracking");
                    self.enabled = false;
                }
            }
        }
    }
}
