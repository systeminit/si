use std::{
    error,
    num::TryFromIntError,
};

use aws_sdk_s3::{
    error::SdkError,
    operation::{
        get_object::GetObjectError,
        head_bucket::HeadBucketError,
        put_object::PutObjectError,
    },
};
use si_data_nats::async_nats::jetstream;
use si_data_pg::{
    PgError,
    PgPoolError,
};
use si_events::{
    ActionId,
    FuncRunId,
    content_hash::ContentHashParseError,
};
use si_std::CanonicalFileError;
use thiserror::Error;
use tokio_stream::Elapsed;

use crate::{
    activities::{
        Activity,
        ActivityId,
    },
    event::LayeredEvent,
    persister::{
        PersistMessage,
        PersisterTaskError,
    },
};

/// S3 operation that failed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S3Operation {
    /// Read operation (get_object)
    Get,
    /// Write operation (put_object)
    Put,
    /// Bucket existence check (head_bucket)
    HeadBucket,
}

impl std::fmt::Display for S3Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S3Operation::Get => write!(f, "Get"),
            S3Operation::Put => write!(f, "Put"),
            S3Operation::HeadBucket => write!(f, "HeadBucket"),
        }
    }
}

/// Wrapper for concrete AWS SDK error types
#[derive(Debug)]
pub enum AwsSdkError {
    /// Error from put_object operation
    PutObject(SdkError<PutObjectError>),
    /// Error from get_object operation
    GetObject(SdkError<GetObjectError>),
    /// Error from head_bucket operation
    HeadBucket(SdkError<HeadBucketError>),
}

impl std::fmt::Display for AwsSdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AwsSdkError::PutObject(e) => write!(f, "{e}"),
            AwsSdkError::GetObject(e) => write!(f, "{e}"),
            AwsSdkError::HeadBucket(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AwsSdkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AwsSdkError::PutObject(e) => Some(e),
            AwsSdkError::GetObject(e) => Some(e),
            AwsSdkError::HeadBucket(e) => Some(e),
        }
    }
}

/// Structured S3 error with categorization and context
#[derive(Error, Debug)]
pub enum S3Error {
    /// Authentication or authorization failure (403, AccessDenied, InvalidAccessKeyId)
    #[error(
        "S3 authentication failed for {operation} (cache: {cache_name}, key: {key}): {message}"
    )]
    Authentication {
        operation: S3Operation,
        cache_name: String,
        key: String,
        message: String,
        #[source]
        source: AwsSdkError,
    },

    /// Resource not found (404, NoSuchBucket, NoSuchKey)
    #[error("S3 resource not found for {operation} (cache: {cache_name}, key: {key}): {message}")]
    NotFound {
        operation: S3Operation,
        cache_name: String,
        key: String,
        message: String,
        #[source]
        source: AwsSdkError,
    },

    /// Rate limiting or throttling (503, SlowDown, RequestLimitExceeded)
    #[error("S3 throttled for {operation} (cache: {cache_name}, key: {key}): {message}")]
    Throttling {
        operation: S3Operation,
        cache_name: String,
        key: String,
        message: String,
        #[source]
        source: AwsSdkError,
    },

    /// Network or connection errors (timeout, dispatch failure)
    #[error("S3 network error for {operation} (cache: {cache_name}, key: {key}): {message}")]
    Network {
        operation: S3Operation,
        cache_name: String,
        key: String,
        message: String,
        #[source]
        source: AwsSdkError,
    },

    /// Configuration errors (construction failure, invalid parameters)
    #[error("S3 configuration error for {operation} (cache: {cache_name}, key: {key}): {message}")]
    Configuration {
        operation: S3Operation,
        cache_name: String,
        key: String,
        message: String,
        #[source]
        source: AwsSdkError,
    },

    /// Uncategorized errors
    #[error("S3 error for {operation} (cache: {cache_name}, key: {key}): {message}")]
    Other {
        operation: S3Operation,
        cache_name: String,
        key: String,
        message: String,
        #[source]
        source: AwsSdkError,
    },
}

impl S3Error {
    /// Get the error kind as a string for metrics/logging
    pub fn kind(&self) -> &'static str {
        match self {
            S3Error::Authentication { .. } => "authentication",
            S3Error::NotFound { .. } => "not_found",
            S3Error::Throttling { .. } => "throttling",
            S3Error::Network { .. } => "network",
            S3Error::Configuration { .. } => "configuration",
            S3Error::Other { .. } => "other",
        }
    }

    /// Get the key from any error variant
    pub fn key(&self) -> &str {
        match self {
            S3Error::Authentication { key, .. }
            | S3Error::NotFound { key, .. }
            | S3Error::Throttling { key, .. }
            | S3Error::Network { key, .. }
            | S3Error::Configuration { key, .. }
            | S3Error::Other { key, .. } => key,
        }
    }

    /// Get the cache name from any error variant
    pub fn cache_name(&self) -> &str {
        match self {
            S3Error::Authentication { cache_name, .. }
            | S3Error::NotFound { cache_name, .. }
            | S3Error::Throttling { cache_name, .. }
            | S3Error::Network { cache_name, .. }
            | S3Error::Configuration { cache_name, .. }
            | S3Error::Other { cache_name, .. } => cache_name,
        }
    }

    /// Get the operation from any error variant
    pub fn operation(&self) -> S3Operation {
        match self {
            S3Error::Authentication { operation, .. }
            | S3Error::NotFound { operation, .. }
            | S3Error::Throttling { operation, .. }
            | S3Error::Network { operation, .. }
            | S3Error::Configuration { operation, .. }
            | S3Error::Other { operation, .. } => *operation,
        }
    }
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum LayerDbError {
    #[error("attempted to find a bunch by action id, but there wasn't one")]
    ActionIdNotFound(ActionId),
    #[error("Activity is not an activity rebase, and should be to be on the work queue")]
    ActivityRebase,
    #[error("Activity Event Server send error: {0}")]
    ActivitySend(#[from] Box<tokio::sync::mpsc::error::SendError<Activity>>),
    #[error(
        "While waiting for an activity id {0}, all senders have closed. The activity will never arrive."
    )]
    ActivityWaitClosed(ActivityId),
    #[error("While waiting for an activity id {0}, the receiving stream has lagged. Cancelling.")]
    ActivityWaitLagged(ActivityId),
    #[error("Timed out waiting for activity id {0} after {1}")]
    ActivityWaitTimeout(ActivityId, Elapsed),
    #[error("AWS config error: {0}")]
    AwsConfig(#[from] si_aws_config::AwsConfigError),
    #[error("cache update message with bad headers: {0}")]
    CacheUpdateBadHeaders(String),
    #[error("cache update message had no headers")]
    CacheUpdateNoHeaders,
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("Configuration validation error: {0}")]
    ConfigValidation(String),
    #[error("content conversion error: {0}")]
    ContentConversion(String),
    #[error("could not convert to key from string")]
    CouldNotConvertToKeyFromString(String),
    #[error("decompression error: {0}")]
    Decompress(String),
    #[error("Foyer error: {0}")]
    Foyer(#[source] Box<dyn error::Error + Sync + Send + 'static>),
    #[error("failed to parse content hash from str: {0}")]
    HashParse(#[from] ContentHashParseError),
    #[error("incomplete key: {0}")]
    IncompleteKey(String),
    #[error("failed to convert integer: {0}")]
    IntConvert(#[from] TryFromIntError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Layered Event Server send error: {0}")]
    LayeredEventSend(#[from] Box<tokio::sync::mpsc::error::SendError<LayeredEvent>>),
    #[error("missing func_run when one was expected: {0}")]
    MissingFuncRun(FuncRunId),
    #[error("missing internal buffer entry when expected; this is an internal bug")]
    MissingInternalBuffer,
    #[error("ack error: {0}")]
    NatsAck(#[source] si_data_nats::async_nats::Error),
    #[error("raw ack error: {0}")]
    NatsAckRaw(String),
    #[error("nats client: {0}")]
    NatsClient(#[from] si_data_nats::Error),
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
    PersisterWriteSend(#[from] Box<tokio::sync::mpsc::error::SendError<PersistMessage>>),
    #[error("pg error: {0}]")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}]")]
    PgPool(#[from] PgPoolError),
    #[error("postcard error: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("failed to create retry queue directory: {0}")]
    RetryQueueDirCreate(#[source] std::io::Error),
    #[error("failed to read retry queue directory: {0}")]
    RetryQueueDirRead(#[source] std::io::Error),
    #[error("failed to delete retry queue file: {0}")]
    RetryQueueFileDelete(#[source] std::io::Error),
    #[error("failed to read retry queue file: {0}")]
    RetryQueueFileRead(#[source] std::io::Error),
    #[error("failed to write retry queue file: {0}")]
    RetryQueueFileWrite(#[source] std::io::Error),
    #[error("invalid retry queue filename: {0:?}")]
    RetryQueueInvalidFilename(std::ffi::OsString),
    #[error("retry queue send error: {0}")]
    RetryQueueSend(String),
    #[error("S3 error: {0}")]
    S3(Box<S3Error>),
    #[error("S3 not configured")]
    S3NotConfigured,
    #[error("S3 write queue error: {0}")]
    S3WriteQueue(String),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
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
