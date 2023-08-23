use crate::environment::Environment;
use futures::StreamExt;
use rabbitmq_stream_client::error::ConsumerDeliveryError;
use rabbitmq_stream_client::types::{Delivery, Message, OffsetSpecification};
use rabbitmq_stream_client::{Consumer as UpstreamConsumer, ConsumerHandle};
use telemetry::prelude::*;
use tokio::task;

use crate::RabbitResult;

/// An interface for consuming RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Consumer {
    inner: UpstreamConsumer,
}

impl Consumer {
    /// Creates a new [`Consumer`] for consuming RabbitMQ stream messages.
    pub async fn new(environment: &Environment, stream: &str) -> RabbitResult<Self> {
        let inner = environment
            .inner()
            .consumer()
            .offset(OffsetSpecification::First)
            .build(stream)
            .await?;
        Ok(Self { inner })
    }

    pub async fn next(&mut self) -> RabbitResult<Option<Result<Delivery, ConsumerDeliveryError>>> {
        Ok(self.inner.next().await)
    }

    pub fn handle(&self) -> ConsumerHandle {
        self.inner.handle()
    }

    pub fn process_delivery(&self, delivery: &Delivery) -> RabbitResult<Option<String>> {
        let maybe_data = delivery
            .message()
            .data()
            .map(|data| String::from_utf8(data.to_vec()));
        Ok(match maybe_data {
            Some(data) => Some(data?),
            None => None,
        })
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
