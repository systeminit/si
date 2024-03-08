use std::error;

use si_data_nats::async_nats::jetstream;
use si_data_pg::{PgError, PgPoolError};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum LayerCacheError {
    #[error("missing internal buffer entry when expected; this is an internal bug")]
    MissingInternalBuffer,
    #[error("error parsing nats message header: {0}")]
    NatsHeaderParse(#[source] Box<dyn error::Error + 'static>),
    #[error("malformed/missing nats headers")]
    NatsMalformedHeaders,
    #[error("nats message missing size header")]
    NatsMissingSizeHeader,
    #[error("error publishing message: {0}")]
    NatsPublish(#[from] jetstream::context::PublishError),
    #[error("error pull message from stream: {0}")]
    NatsPullMessages(#[from] jetstream::consumer::pull::MessagesError),
    #[error("pg error: {0}]")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}]")]
    PgPool(#[from] PgPoolError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("sled error: {0}")]
    SledError(#[from] sled::Error),
}

impl LayerCacheError {
    pub fn nats_header_parse<E>(err: E) -> Self
    where
        E: error::Error + 'static,
    {
        Self::NatsHeaderParse(Box::new(err))
    }
}

pub type LayerCacheResult<T> = Result<T, LayerCacheError>;
