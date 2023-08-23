use rabbitmq_stream_client::types::Message;
use rabbitmq_stream_client::{Dedup, NoDedup, Producer as UpstreamProducer};
use telemetry::prelude::warn;
use tokio::task;

use crate::environment::Environment;
use crate::{RabbitError, RabbitResult};

/// An interface for producing and sending RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Producer {
    inner: UpstreamProducer<Dedup>,
    closed: bool,
}

impl Producer {
    /// Creates a new [`Producer`] for producing and sending RabbitMQ stream messages.
    pub async fn new(
        environment: &Environment,
        name: impl AsRef<str>,
        stream: impl AsRef<str>,
    ) -> RabbitResult<Self> {
        let inner = environment
            .inner()
            .producer()
            .name(name.as_ref())
            .build(stream.as_ref())
            .await?;
        Ok(Self {
            inner,
            closed: false,
        })
    }

    /// Sends a single message to a stream.
    pub async fn send_single(&mut self, message: impl Into<Vec<u8>>) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        self.inner
            .send_with_confirm(Message::builder().body(message).build())
            .await?;
        Ok(())
    }

    /// Sends a batch of messages to a stream.
    pub async fn send_batch(&mut self, messages: Vec<impl Into<Vec<u8>>>) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        self.inner
            .batch_send_with_confirm(
                messages
                    .into_iter()
                    .map(|m| Message::builder().body(m.into()).build())
                    .collect(),
            )
            .await?;
        Ok(())
    }

    // Closes the producer connection and renders the producer unusable.
    pub async fn close(mut self) -> RabbitResult<()> {
        self.inner.close().await?;
        self.closed = true;
        Ok(())
    }
}
