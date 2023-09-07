use rabbitmq_stream_client::types::Delivery as UpstreamDelivery;
use serde_json::Value;

use crate::RabbitError;

/// This type is a deconstruction of the upstream
/// [`Delivery`](rabbitmq_stream_client::types::Delivery) type.
#[derive(Debug, Clone)]
pub struct Delivery {
    /// The contents of the message.
    pub message_contents: Option<Value>,
    /// The contents of the "reply_to" field from the message properties.
    pub reply_to: Option<String>,
}

impl TryFrom<UpstreamDelivery> for Delivery {
    type Error = RabbitError;

    fn try_from(value: UpstreamDelivery) -> Result<Self, Self::Error> {
        let message = value.message();

        let contents: Option<Value> = match message.data() {
            Some(data) => serde_json::from_slice(data)?,
            None => None,
        };

        let reply_to = match message.properties() {
            Some(properties) => properties.reply_to.clone(),
            None => None,
        };

        Ok(Self {
            message_contents: contents,
            reply_to,
        })
    }
}
