use rabbitmq_stream_client::types::Message;
use rabbitmq_stream_client::{NoDedup, Producer as UpstreamProducer};
use serde::Serialize;

use crate::environment::Environment;
use crate::{RabbitError, RabbitResult};

/// An interface for producing and sending RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Producer {
    inner: UpstreamProducer<NoDedup>,
    closed: bool,
}

impl Producer {
    /// Creates a new [`Producer`] for producing and sending RabbitMQ stream messages.
    pub async fn new(environment: &Environment, stream: impl AsRef<str>) -> RabbitResult<Self> {
        let inner = environment
            .inner()
            .producer()
            .build(stream.as_ref())
            .await?;
        Ok(Self {
            inner,
            closed: false,
        })
    }

    /// Sends a single message to a stream.
    pub async fn send_single<T: Serialize>(
        &mut self,
        input: T,
        reply_to: Option<String>,
    ) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        let value = serde_json::to_value(input)?;
        let mut message_builder = Message::builder().body(serde_json::to_vec(&value)?);
        if let Some(reply_to) = reply_to {
            message_builder = message_builder
                .properties()
                .reply_to(reply_to)
                .message_builder();
        }
        let message = message_builder.build();

        self.inner.send_with_confirm(message).await?;
        Ok(())
    }

    /// Closes the producer connection and renders the producer unusable.
    pub async fn close(mut self) -> RabbitResult<()> {
        self.inner.close().await?;
        self.closed = true;
        Ok(())
    }
}
