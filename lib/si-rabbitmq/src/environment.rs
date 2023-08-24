use rabbitmq_stream_client::error::{StreamCreateError, StreamDeleteError};
use rabbitmq_stream_client::types::{ByteCapacity, ResponseCode};
use rabbitmq_stream_client::Environment as UpstreamEnvironment;

use crate::error::RabbitResult;

const STREAM_LENGTH_CAPACTIY_IN_MEGABYTES: u64 = 10;

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

    /// Attempts to create the stream and returns a boolean indicates if the stream was actually
    /// created (i.e. "false" if it already exists).
    pub async fn create_stream(&self, stream: impl AsRef<str>) -> RabbitResult<bool> {
        match self
            .inner
            .stream_creator()
            .max_length(ByteCapacity::MB(STREAM_LENGTH_CAPACTIY_IN_MEGABYTES))
            .create(stream.as_ref())
            .await
        {
            Ok(()) => Ok(false),
            Err(e) => match e {
                StreamCreateError::Create {
                    status: ResponseCode::StreamAlreadyExists,
                    stream: _,
                } => Ok(true),
                e => Err(e.into()),
            },
        }
    }

    /// Attempts to delete the stream and returns a boolean indicates if the stream was actually
    /// deleted (i.e. "false" if it does not currently exist).
    pub async fn delete_stream(&self, stream: impl AsRef<str>) -> RabbitResult<bool> {
        match self.inner.delete_stream(stream.as_ref()).await {
            Ok(()) => Ok(true),
            Err(e) => match e {
                StreamDeleteError::Delete {
                    status: ResponseCode::StreamDoesNotExist,
                    stream: _,
                } => Ok(false),
                e => Err(e.into()),
            },
        }
    }
}
