mod app_state;
mod config;
mod handlers;
mod publisher;
mod request;
mod server;

use std::io;

use si_data_nats::{async_nats, NatsError, Subject};
use thiserror::Error;

pub use si_pool_noodle::{instance::cyclone::LocalUdsInstance, Instance};

pub(crate) use crate::publisher::{Publisher, PublisherError};
pub use crate::{
    config::{
        detect_and_configure_development, Config, ConfigBuilder, ConfigError, ConfigFile,
        CycloneSpec, CycloneStream, StandardConfig, StandardConfigFile,
    },
    server::Server,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("cyclone pool error: {0}")]
    CyclonePool(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone spec setup error: {0}")]
    CycloneSetupError(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("jetstream consumer error: {0}")]
    JetStreamConsumer(#[from] async_nats::jetstream::stream::ConsumerError),
    #[error("jetstream consumer stream error: {0}")]
    JetStreamConsumerStream(#[from] async_nats::jetstream::consumer::StreamError),
    #[error("jetstream create stream error: {0}")]
    JetStreamCreateStreamError(#[from] async_nats::jetstream::context::CreateStreamError),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("failed to initialize a nats client: {0}")]
    NatsClient(#[source] NatsError),
    #[error("failed to subscribe to nats subject ({0}): {1}")]
    NatsSubscribe(Subject, #[source] NatsError),
    #[error("naxum error: {0}")]
    Naxum(#[source] io::Error),
    #[error("veritech decryption key error: {0}")]
    VeritechDecryptionKey(#[from] si_crypto::VeritechDecryptionKeyError),
    #[error("wrong cyclone spec type for {0} spec: {1:?}")]
    WrongCycloneSpec(&'static str, Box<CycloneSpec>),
}

type ServerResult<T> = Result<T, ServerError>;
