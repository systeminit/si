use std::time::Duration;

use reqwest::{self, StatusCode};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    api::{PosthogApi, PosthogApiEvent},
    error::{PosthogError, PosthogResult},
};

#[derive(Debug)]
pub struct PosthogSender {
    rx: UnboundedReceiver<PosthogApi>,
    api_endpoint: String,
    api_key: String,
    reqwest: reqwest::Client,
    enabled: bool,
}

impl PosthogSender {
    pub fn new(
        rx: UnboundedReceiver<PosthogApi>,
        api_endpoint: String,
        api_key: String,
        timeout: Duration,
    ) -> PosthogResult<PosthogSender> {
        let client = reqwest::Client::builder().timeout(timeout).build()?;

        Ok(PosthogSender {
            rx,
            api_endpoint,
            api_key,
            reqwest: client,
            enabled: true,
        })
    }

    pub async fn run(mut self) {
        tracing::info!("PostHog Sender running.");
        while let Some(msg) = self.rx.recv().await {
            if self.enabled {
                tracing::debug!("sending message to posthog: {:?}", &msg);
                let result = match msg {
                    PosthogApi::Event(e) => self.send_event(e).await,
                    PosthogApi::Disable => {
                        tracing::debug!("disabling posthog tracking");
                        self.enabled = false;
                        Ok(())
                    }
                };
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("error sending to posthog: {}", e.to_string());
                    }
                }
            }
        }
    }

    pub async fn send_event(&self, event: PosthogApiEvent) -> PosthogResult<()> {
        let mut event_json = serde_json::to_value(event)?;
        event_json
            .as_object_mut()
            .unwrap()
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
