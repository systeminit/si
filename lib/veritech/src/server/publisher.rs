use cyclone::resolver_function::{FunctionResult, OutputStream};
use serde::Serialize;
use si_data::NatsClient;
use thiserror::Error;

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
    reply_mailbox: &'a str,
}

impl<'a> Publisher<'a> {
    pub fn new(nats: &'a NatsClient, reply_mailbox: &'a str) -> Self {
        Self {
            nats,
            reply_mailbox,
        }
    }

    pub async fn publish(&self, message: &impl Publishable) -> Result<()> {
        let nats_msg = serde_json::to_string(message).map_err(PublisherError::JSONSerialize)?;

        self.nats
            .publish(self.reply_mailbox, nats_msg)
            .await
            .map_err(|err| PublisherError::NatsPublish(err, self.reply_mailbox.to_string()))
    }
}

pub trait Publishable: Serialize {}

impl Publishable for OutputStream {}
impl Publishable for FunctionResult {}
