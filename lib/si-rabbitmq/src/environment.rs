use rabbitmq_stream_client::error::{StreamCreateError, StreamDeleteError};
use rabbitmq_stream_client::types::{ByteCapacity, ResponseCode};
use rabbitmq_stream_client::Environment as UpstreamEnvironment;
use std::time::Duration;
use telemetry::prelude::{info, trace, warn};

use crate::{config::Config, error::RabbitResult, RabbitError};

const STREAM_LENGTH_CAPACTIY_IN_MEGABYTES: u64 = 10;
const ENVIRONMENT_CREATION_TIMEOUT_DURATION_IN_SECONDS: u64 = 4;
const ENVIRONMENT_CREATION_RETRIES: u64 = 3;

/// A connection to a RabbitMQ node.
#[allow(missing_debug_implementations)]
pub struct Environment {
    inner: UpstreamEnvironment,
}

impl Environment {
    /// Creates a new [`Environment`], which contains a connection to a RabbitMQ node.
    pub async fn new(config: &Config) -> RabbitResult<Self> {
        // Create a range starting from "1" using the retries constant.
        let range = 1..ENVIRONMENT_CREATION_RETRIES + 1;

        // Attempt to create an environment with the above range and the given timeout duration.
        for attempt in range {
            info!("attempt {attempt} of {ENVIRONMENT_CREATION_RETRIES} to connect to RabbitMQ...");
            match tokio::time::timeout(
                Duration::from_secs(ENVIRONMENT_CREATION_TIMEOUT_DURATION_IN_SECONDS),
                UpstreamEnvironment::builder()
                    .host(config.host())
                    .username(config.username())
                    .password(config.password())
                    .port(config.port())
                    .build(),
            )
            .await
            {
                Ok(result) => return Ok(Self { inner: result? }),
                Err(elapsed) => {
                    warn!("hit timeout when trying to communicate with RabbitMQ (duration: {ENVIRONMENT_CREATION_TIMEOUT_DURATION_IN_SECONDS} seconds)");
                    trace!("{elapsed}");
                }
            };
        }

        // If we have exited the loop, we have hit the max number of retries.
        Err(RabbitError::EnvironmentCreationFailed)
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
