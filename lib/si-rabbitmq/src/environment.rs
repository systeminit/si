use rabbitmq_stream_client::error::StreamDeleteError;
use rabbitmq_stream_client::types::{ByteCapacity, ResponseCode};
use rabbitmq_stream_client::Environment as UpstreamEnvironment;

use crate::error::RabbitResult;

/// A connection to a RabbitMQ node.
#[allow(missing_debug_implementations)]
pub struct Environment {
    inner: UpstreamEnvironment,
}

impl Environment {
    /// Creates a new [`Environment`], which contains a connection to a RabbitMQ node.
    pub async fn new() -> RabbitResult<Self> {
        let inner = UpstreamEnvironment::builder()
            .host("localhost")
            .username("guest")
            .password("guest")
            .port(5552)
            .build()
            .await?;
        Ok(Self { inner })
    }

    /// Returns the inner data structure handling the connection.
    pub fn inner(&self) -> &UpstreamEnvironment {
        &self.inner
    }

    pub async fn create_stream(&self, stream: impl AsRef<str>) -> RabbitResult<()> {
        Ok(self
            .inner
            .stream_creator()
            .max_length(ByteCapacity::KB(400))
            .create(stream.as_ref())
            .await?)
    }

    pub async fn delete_stream(&self, stream: impl AsRef<str>) -> RabbitResult<()> {
        match self.inner.delete_stream(stream.as_ref()).await {
            Ok(()) => Ok(()),
            Err(e) => match e {
                StreamDeleteError::Delete {
                    status: ResponseCode::StreamDoesNotExist,
                    stream: _,
                } => Ok(()),
                e => Err(e.into()),
            },
        }
    }
}
