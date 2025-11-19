#![recursion_limit = "256"]

use thiserror::Error;

pub mod api_types;
mod app_state;
mod change_set_processor_task;
pub mod compressing_stream;
mod config;
mod deployment_processor_task;
mod handlers;
mod local_message;
mod server;
pub use si_service_endpoints::{
    DefaultServiceEndpoints,
    ServiceEndpointsConfig,
    server::EndpointsServer,
};
pub use si_settings::{
    ConfigMap,
    ParameterProvider,
};
mod materialized_view;
pub(crate) mod updates;

pub use self::{
    config::{
        Config,
        ConfigError,
        ConfigFile,
        StandardConfigFile,
        detect_and_configure_development,
    },
    server::{
        Server,
        ServerMetadata,
    },
};

/// An error that can be returned when an Edda service is running
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    /// When a compute executor fails to initialize
    #[error("compute executor initialization error: {0}")]
    ComputeExecutorInitialize(#[from] dal::DedicatedExecutorInitializeError),
    /// When a Cyclone encryption key failed to be loaded
    #[error("error when loading encryption key: {0}")]
    CycloneEncryptionKey(#[source] si_crypto::VeritechEncryptionKeyError),
    /// When the DAL library fails to be initialized
    #[error("dal initialization error: {0}")]
    DalInitialization(#[from] dal::InitializationError),
    /// When we fail to get or create inner NATS Jetstream streams
    #[error("dal jetstream streams error: {0}")]
    DalJetstreamStreams(#[from] dal::JetstreamStreamsError),
    /// When a job queue processor fails to be created
    #[error("job queue processor error: {0}")]
    DalJobQueueProcessor(#[from] dal::job::processor::JobQueueProcessorError),
    /// When a database pool error occurs
    #[error("dal pg pool error: {0}")]
    DalPgPool(#[source] Box<si_data_pg::PgPoolError>),
    /// When failing to create or fetch a Jetstream consumer
    #[error("jetstream consumer error: {0}")]
    JsConsumer(#[from] si_data_nats::async_nats::jetstream::stream::ConsumerError),
    /// When failing to create a Jetstream consumer `impl Stream` of messages
    #[error("consumer stream error: {0}")]
    JsConsumerStream(#[from] si_data_nats::async_nats::jetstream::consumer::StreamError),
    /// When failing to create a Jetstream stream
    #[error("stream create error: {0}")]
    JsCreateStreamError(#[from] si_data_nats::async_nats::jetstream::context::CreateStreamError),
    /// When we fail to create or fetch a jetstream k/v store
    #[error("kv store error: {0}")]
    KvStore(#[from] frigg::FriggError),
    /// When a LayerDb error occurs
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    /// When a NATS client fails to be created successfully
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
    /// When a naxum-based service encounters an I/O error
    #[error("naxum error: {0}")]
    Naxum(#[source] std::io::Error),
    /// When a rebaser client error occurs
    #[error("rebaser client error: {0}")]
    Rebaser(#[from] rebaser_client::ClientError),
    /// When a symmetric crypto service fails to be created
    #[error("symmetric crypto error: {0}")]
    SymmetricCrypto(#[from] si_crypto::SymmetricCryptoError),
}

impl ServerError {
    /// Converts a pg pool error into a server error.
    pub fn dal_pg_pool(err: si_data_pg::PgPoolError) -> Self {
        Self::DalPgPool(Box::new(err))
    }
}

type Error = ServerError;

type Result<T> = std::result::Result<T, ServerError>;
