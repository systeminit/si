use rabbitmq_stream_client::types::Message;
use rabbitmq_stream_client::{Dedup, NoDedup, Producer as UpstreamProducer};

use crate::connection::StreamManager;
use crate::{RabbitError, RabbitResult};

/// An interface for producing and sending RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Producer {
    producer: UpstreamProducer<Dedup>,
    closed: bool,
}

impl Producer {
    /// Creates a new [`Producer`] for producing and sending RabbitMQ stream messages.
    pub async fn new(
        connection: &StreamManager,
        name: impl AsRef<str>,
        stream: impl AsRef<str>,
    ) -> RabbitResult<Self> {
        let producer = connection
            .inner()
            .producer()
            .name(name.as_ref())
            .build(stream.as_ref())
            .await?;
        Ok(Self {
            producer,
            closed: false,
        })
    }

    /// Sends a single message to a stream.
    pub async fn send_single(&mut self, message: impl Into<Vec<u8>>) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        self.producer
            .send_with_confirm(Message::builder().body(message).build())
            .await?;
        Ok(())
    }

    /// Sends a batch of messages to a stream.
    pub async fn send_batch(&mut self, messages: Vec<impl Into<Vec<u8>>>) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        self.producer
            .batch_send_with_confirm(
                messages
                    .into_iter()
                    .map(|m| Message::builder().body(m.into()).build())
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
