use crate::connection::Connection;
use futures::StreamExt;
use rabbitmq_stream_client::error::ConsumerDeliveryError;
use rabbitmq_stream_client::types::Delivery;
use rabbitmq_stream_client::Consumer as UpstreamConsumer;
use tokio::task;

use crate::RabbitResult;

/// An interface for consuming RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Consumer(UpstreamConsumer);

impl Consumer {
    /// Creates a new [`Consumer`] for consuming RabbitMQ stream messages.
    pub async fn new(connection: &Connection, stream: &str) -> RabbitResult<Self> {
        let consumer = connection.inner().consumer().build(stream).await?;
        Ok(Self(consumer))
    }

    /// Starts a consumer task that watches the stream.
    pub async fn start(
        mut self,
        processing_func: fn(delivery: Result<Delivery, ConsumerDeliveryError>),
    ) -> RabbitResult<()> {
        let handle = self.0.handle();
        task::spawn(async move {
            while let Some(delivery) = self.0.next().await {
                processing_func(delivery)
            }
        });

        // TODO(nick): handle when close happens more precisely.
        handle.close().await?;
        Ok(())
    }
}
