use serde::Serialize;
use si_data_nats::Subject;
use si_pool_noodle::{FunctionResult, OutputStream};
use telemetry::tracing::warn;
use telemetry_nats::propagation;
use thiserror::Error;
use veritech_core::{reply_mailbox_for_output, reply_mailbox_for_result, FINAL_MESSAGE_HEADER_KEY};

use crate::nats_client::NatsClient;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("failed to publish message to nats subject: {1}")]
    NatsPublish(#[source] si_data_nats::NatsError, String),
}

type Result<T> = std::result::Result<T, PublisherError>;

#[derive(Debug)]
pub struct Publisher {
    nats: NatsClient,
    reply_mailbox_output: Subject,
    reply_mailbox_result: Subject,
}

impl Publisher {
    pub fn new(nats: NatsClient, reply_mailbox: &str) -> Self {
        Self {
            nats,
            reply_mailbox_output: reply_mailbox_for_output(reply_mailbox).into(),
            reply_mailbox_result: reply_mailbox_for_result(reply_mailbox).into(),
        }
    }

    pub async fn publish_output(&self, output: &OutputStream) -> Result<()> {
        let nats_msg = serde_json::to_string(output).map_err(PublisherError::JSONSerialize)?;

        loop {
            let guard = self.nats.lock().await;
            let timeout_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(2),
                guard.publish_with_headers(
                    self.reply_mailbox_output.clone(),
                    propagation::empty_injected_headers(),
                    nats_msg.clone().into(),
                ),
            )
            .await;
            drop(guard);
            match timeout_result {
                Ok(publish_result) => publish_result.map_err(|err| {
                    PublisherError::NatsPublish(err, self.reply_mailbox_output.to_string())
                })?,
                Err(_) => {
                    warn!("publisher: dropping guard and sleeping to give time for the hot swapped client...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    pub async fn finalize_output(&self) -> Result<()> {
        let mut headers = si_data_nats::HeaderMap::new();
        headers.insert(FINAL_MESSAGE_HEADER_KEY, "true");
        propagation::inject_headers(&mut headers);

        loop {
            let guard = self.nats.lock().await;
            let timeout_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(2),
                guard.publish_with_headers(
                    self.reply_mailbox_output.clone(),
                    headers.clone(),
                    vec![].into(),
                ),
            )
            .await;
            drop(guard);
            match timeout_result {
                Ok(publish_result) => publish_result.map_err(|err| {
                    PublisherError::NatsPublish(err, self.reply_mailbox_output.to_string())
                })?,
                Err(_) => {
                    warn!("publisher: dropping guard and sleeping to give time for the hot swapped client...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    pub async fn publish_result<R>(&self, result: &FunctionResult<R>) -> Result<()>
    where
        R: Serialize,
    {
        let nats_msg = serde_json::to_string(result).map_err(PublisherError::JSONSerialize)?;

        loop {
            let guard = self.nats.lock().await;
            let timeout_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(2),
                guard.publish_with_headers(
                    self.reply_mailbox_result.clone(),
                    propagation::empty_injected_headers(),
                    nats_msg.clone().into(),
                ),
            )
            .await;
            drop(guard);
            match timeout_result {
                Ok(publish_result) => publish_result.map_err(|err| {
                    PublisherError::NatsPublish(err, self.reply_mailbox_output.to_string())
                })?,
                Err(_) => {
                    warn!("publisher: dropping guard and sleeping to give time for the hot swapped client...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }
}
