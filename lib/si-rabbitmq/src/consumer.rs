use futures::StreamExt;
use rabbitmq_stream_client::types::OffsetSpecification;
use rabbitmq_stream_client::{
    Consumer as UpstreamConsumer, ConsumerHandle as UpstreamConsumerHandle,
};
use telemetry::prelude::*;
use tokio::task;

use crate::Delivery;
use crate::Environment;
use crate::RabbitResult;

/// A type alias to the upstream [`ConsumerHandle`](rabbitmq_stream_client::ConsumerHandle).
pub type ConsumerHandle = UpstreamConsumerHandle;

/// A type alias to the upstream [`OffsetSpecification`](OffsetSpecification).
pub type ConsumerOffsetSpecification = OffsetSpecification;

/// An interface for consuming RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Consumer {
    stream: String,
    inner: UpstreamConsumer,
}

impl Consumer {
    /// Creates a new [`Consumer`] for consuming RabbitMQ stream messages.
    pub async fn new(
        environment: &Environment,
        stream: impl Into<String>,
        offset_specification: ConsumerOffsetSpecification,
    ) -> RabbitResult<Self> {
        let stream = stream.into();
        let inner = environment
            .inner()
            .consumer()
            .offset(offset_specification)
            .build(&stream)
            .await?;
        Ok(Self { stream, inner })
    }

    /// A wrapper around the upstream stream polling implementation.
    pub async fn next(&mut self) -> RabbitResult<Option<Delivery>> {
        if let Some(unprocessed_delivery) = self.inner.next().await {
            let delivery = unprocessed_delivery?;
            return Ok(Some(Delivery::try_from(delivery)?));
        }
        Ok(None)
    }

    /// Provides a [`ConsumerHandle`].
    pub fn handle(&self) -> ConsumerHandle {
        self.inner.handle()
    }

    /// Returns the stream name for the [`Consumer`](Consumer).
    pub fn stream(&self) -> &String {
        &self.stream
    }
}

impl Drop for Consumer {
    fn drop(&mut self) {
        let handle = self.handle();

        // Close the consumer associated to the handle provided.
        task::spawn(async {
            if let Err(e) = handle.close().await {
                warn!("error when closing consumer on drop: {e}");
            }
        });
    }
}
