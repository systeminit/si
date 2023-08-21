use rabbitmq_stream_client::types::ByteCapacity;
use rabbitmq_stream_client::Environment;

use crate::error::RabbitResult;

/// A connection to a RabbitMQ node.
#[allow(missing_debug_implementations)]
pub struct Connection {
    environment: Environment,
}

impl Connection {
    /// Creates a new [`Connection`], which contains a connection to a RabbitMQ node.
    pub async fn new() -> RabbitResult<Self> {
        let environment = Environment::builder()
            .host("localhost")
            .port(5672)
            .build()
            .await?;
        Ok(Self { environment })
    }

    /// Returns the inner data structure handling the connection.
    pub fn inner(&self) -> &Environment {
        &self.environment
    }

    pub async fn create_stream(&self, stream: impl AsRef<str>) -> RabbitResult<()> {
        Ok(self
            .environment
            .stream_creator()
            .max_length(ByteCapacity::KB(400))
            .create(stream.as_ref())
            .await?)
    }

    pub async fn delete_stream(&self, stream: impl AsRef<str>) -> RabbitResult<()> {
        Ok(self.environment.delete_stream(stream.as_ref()).await?)
    }
}
