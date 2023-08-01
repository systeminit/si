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
        let environment = Environment::builder().build().await?;
        Ok(Self { environment })
    }

    /// Returns the inner data structure handling the connection.
    pub fn inner(&self) -> &Environment {
        &self.environment
    }
}
