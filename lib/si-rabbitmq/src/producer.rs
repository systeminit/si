use rabbitmq_stream_client::types::Message;
use rabbitmq_stream_client::{NoDedup, Producer as UpstreamProducer};

use crate::connection::Connection;
use crate::{RabbitError, RabbitResult};

/// An interface for producing and sending RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Producer {
    producer: UpstreamProducer<NoDedup>,
    closed: bool,
}

impl Producer {
    /// Creates a new [`Producer`] for producing and sending RabbitMQ stream messages.
    pub async fn new(connection: &Connection, stream: &str) -> RabbitResult<Self> {
        let producer = connection.inner().producer().build(stream).await?;
        Ok(Self {
            producer,
            closed: false,
        })
    }

    /// Sends a single message to a stream.
    pub async fn send_single(&self, message: impl Into<Vec<u8>>) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        self.producer
            .send_with_confirm(Message::builder().body(message).build())
            .await?;
        Ok(())
    }

    /// Sends a batch of messages to a stream.
    pub async fn send_batch(&self, messages: impl Into<Vec<Vec<u8>>>) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        self.producer
            .batch_send_with_confirm(
                messages
                    .into()
                    .iter()
                    .map(|m| Message::builder().body(m.clone()).build())
                    .collect(),
            )
            .await?;
        Ok(())
    }

    /// Closes the producer connection and renders the producer unusable.
    pub async fn close(mut self) -> RabbitResult<()> {
        self.producer.close().await?;
        self.closed = true;
        Ok(())
    }
}
