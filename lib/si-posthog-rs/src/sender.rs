use std::time::Duration;

use reqwest::{self, StatusCode};
use telemetry::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    api::{PosthogApiEvent, PosthogMessage},
    error::{PosthogError, PosthogResult},
    PosthogConfig,
};

#[derive(Debug)]
pub struct PosthogSender {
    rx: UnboundedReceiver<PosthogMessage>,
    api_endpoint: String,
    api_key: String,
    reqwest: reqwest::Client,
    enabled: bool,
}

impl PosthogSender {
    pub(crate) fn new(
        rx: UnboundedReceiver<PosthogMessage>,
        config: &PosthogConfig,
    ) -> PosthogResult<PosthogSender> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.request_timeout_ms()))
            .build()?;

        Ok(PosthogSender {
            rx,
            api_endpoint: config.api_endpoint().to_string(),
            api_key: config.api_key().to_string(),
            reqwest: client,
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
                        if let Err(err) = self.send_event(event).await {
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

    pub async fn send_event(&self, event: PosthogApiEvent) -> PosthogResult<()> {
        let mut event_json = serde_json::to_value(event)?;
        event_json
            .as_object_mut()
            .expect("event was explicitly serialized from rust type as is therefore an object")
            .insert("api_key".to_string(), serde_json::json!(self.api_key));

        let response = self
            .reqwest
            .post(format!(
                "{api_endpoint}/capture",
                api_endpoint = self.api_endpoint
            ))
            .json(&event_json)
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(()),
            error => Err(PosthogError::PosthogApi(error, response.text().await?)),
        }
    }
}
