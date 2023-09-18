use telemetry::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::api::PosthogApiClient;
use crate::{api::PosthogMessage, error::PosthogResult, PosthogConfig};

#[derive(Debug)]
pub struct PosthogSender {
    rx: UnboundedReceiver<PosthogMessage>,
    api_client: PosthogApiClient,
    enabled: bool,
}

impl PosthogSender {
    pub(crate) fn new(
        rx: UnboundedReceiver<PosthogMessage>,
        config: &PosthogConfig,
    ) -> PosthogResult<PosthogSender> {
        let api_client = PosthogApiClient::new(config)?;

        Ok(PosthogSender {
            rx,
            api_client,
            enabled: config.enabled(),
        })
    }

    pub async fn run(mut self) {
        debug!("PostHog Sender running.");
        if !self.enabled {
            debug!("posthog tracking is disabled");
        }

        while let Some(msg) = self.rx.recv().await {
            trace!(message = ?msg, "received message");
            if self.enabled {
                match msg {
                    PosthogMessage::Event(event) => {
                        debug!(event = ?event, "sending event to posthog");
                        if let Err(err) = self.api_client.send_event(event).await {
                            error!(error = ?err, "error sending event to posthog");
                        }
                    }
                    PosthogMessage::Disable => {
                        debug!("disabling posthog tracking");
                        self.enabled = false;
                    }
                }
            }
        }
    }
}
