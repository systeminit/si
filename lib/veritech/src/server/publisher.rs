use cyclone::FunctionResult;
use deadpool_cyclone::OutputStream;
use serde::Serialize;
use si_data::NatsClient;
use thiserror::Error;

use crate::{reply_mailbox_for_output, reply_mailbox_for_result, FINAL_MESSAGE_HEADER_KEY};

#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("failed to publish message to nats subject: {1}")]
    NatsPublish(#[source] si_data::NatsError, String),
}

type Result<T> = std::result::Result<T, PublisherError>;

#[derive(Debug)]
pub struct Publisher<'a> {
    nats: &'a NatsClient,
    reply_mailbox_output: String,
    reply_mailbox_result: String,
}

impl<'a> Publisher<'a> {
    pub fn new(nats: &'a NatsClient, reply_mailbox: &str) -> Self {
        Self {
            nats,
            reply_mailbox_output: reply_mailbox_for_output(reply_mailbox),
            reply_mailbox_result: reply_mailbox_for_result(reply_mailbox),
        }
    }

    pub async fn publish_output(&self, output: &OutputStream) -> Result<()> {
        let nats_msg = serde_json::to_string(output).map_err(PublisherError::JSONSerialize)?;

        self.nats
            .publish(&self.reply_mailbox_output, nats_msg)
            .await
            .map_err(|err| PublisherError::NatsPublish(err, self.reply_mailbox_output.clone()))
    }

    pub async fn finalize_output(&self) -> Result<()> {
        let headers = [(FINAL_MESSAGE_HEADER_KEY, "true")].iter().collect();
        self.nats
            .publish_with_reply_or_headers(
                &self.reply_mailbox_output,
                None::<String>,
                Some(&headers),
                vec![],
            )
            .await
            .map_err(|err| PublisherError::NatsPublish(err, self.reply_mailbox_output.clone()))
    }

    pub async fn publish_result<R>(&self, result: &FunctionResult<R>) -> Result<()>
    where
        R: Serialize,
    {
        let nats_msg = serde_json::to_string(result).map_err(PublisherError::JSONSerialize)?;

        self.nats
            .publish(&self.reply_mailbox_result, nats_msg)
            .await
            .map_err(|err| PublisherError::NatsPublish(err, self.reply_mailbox_result.clone()))
    }
}
