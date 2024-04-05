use std::error;

use si_data_nats::async_nats::jetstream;
use si_data_pg::{PgError, PgPoolError};
use si_events::ContentHashParseError;
use si_std::CanonicalFileError;
use thiserror::Error;
use tokio_stream::Elapsed;

use crate::{
    activities::ActivityId,
    persister::{PersistMessage, PersisterTaskError},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum LayerDbError {
    #[error("While waiting for an activity id {0}, all senders have closed. The activity will never arrive.")]
    ActivityWaitClosed(ActivityId),
    #[error("While waiting for an activity id {0}, the receiving stream has lagged. Cancelling.")]
    ActivityWaitLagged(ActivityId),
    #[error("Timed out waiting for activity id {0} after {1}")]
    ActivityWaitTimeout(ActivityId, Elapsed),
    #[error("cache update message with bad headers: {0}")]
    CacheUpdateBadHeaders(String),
    #[error("cache update message had no headers")]
    CacheUpdateNoHeaders,
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("content conversion error: {0}")]
    ContentConversion(String),
    #[error("could not convert to key from string")]
    CouldNotConvertToKeyFromString(String),
    #[error("failed to parse content hash from str: {0}")]
    HashParse(#[from] ContentHashParseError),
    #[error("invalid cache name: {0}")]
    InvalidCacheName(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("missing internal buffer entry when expected; this is an internal bug")]
    MissingInternalBuffer,
    #[error("ack error: {0}")]
    NatsAck(#[source] si_data_nats::async_nats::Error),
    #[error("stream consumer error: {0}")]
    NatsConsumer(#[from] jetstream::stream::ConsumerError),
    #[error("error while fetching or creating a nats jetsream: {0}")]
    NatsCreateStream(#[from] jetstream::context::CreateStreamError),
    #[error("error parsing nats message header: {0}")]
    NatsHeaderParse(#[source] Box<dyn error::Error + Send + Sync + 'static>),
    #[error("malformed/missing nats headers")]
    NatsMalformedHeaders,
    #[error("nats message missing size header")]
    NatsMissingSizeHeader,
    #[error("error publishing message: {0}")]
    NatsPublish(#[from] jetstream::context::PublishError),
    #[error("error pull message from stream: {0}")]
    NatsPullMessages(#[from] jetstream::consumer::pull::MessagesError),
    #[error("consumer stream error: {0}")]
    NatsStream(#[from] jetstream::consumer::StreamError),
    #[error("persister task write failed: {0:?}")]
    PersisterTaskFailed(PersisterTaskError),
    #[error("persister write error: {0}")]
    PersisterWriteSend(#[from] tokio::sync::mpsc::error::SendError<PersistMessage>),
    #[error("pg error: {0}]")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}]")]
    PgPool(#[from] PgPoolError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("sled error: {0}")]
    SledError(#[from] sled::Error),
    #[error("tokio oneshot recv error: {0}")]
    TokioOneShotRecv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("unexpected activity variant; expected={0}, actual={1}")]
    UnexpectedActivityVariant(String, String),
}

impl LayerDbError {
    pub fn nats_header_parse<E>(err: E) -> Self
    where
        E: error::Error + Send + Sync + 'static,
    {
        Self::NatsHeaderParse(Box::new(err))
    }
}

pub type LayerDbResult<T> = Result<T, LayerDbError>;
