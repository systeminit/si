#![recursion_limit = "256"]
#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

use std::io;

use audit_database::AuditDatabaseContextError;
use frigg::FriggError;
use si_data_spicedb::SpiceDbError;
use thiserror::Error;

mod app;
mod app_state;
mod config;
mod extract;
mod garbage_collection;
mod init;
pub mod key_generation;
pub mod middleware;
mod migrations;
mod nats_multiplexer;
mod routes;
mod runnable;
mod server;
pub mod service;
mod tracking;
mod uds;

pub use dal::{
    JobQueueProcessor,
    NatsProcessor,
    ServicesContext,
    feature_flags::{
        FeatureFlag,
        FeatureFlagService,
    },
};
pub use si_settings::{
    ConfigMap,
    ParameterProvider,
};

pub use self::{
    app::AxumApp,
    app_state::ApplicationRuntimeMode,
    config::{
        Config,
        ConfigBuilder,
        ConfigError,
        ConfigFile,
        IncomingStream,
        MigrationMode,
        StandardConfig,
        StandardConfigFile,
        WorkspacePermissions,
        WorkspacePermissionsMode,
    },
    garbage_collection::SnapshotGarbageCollector,
    migrations::Migrator,
    nats_multiplexer::CRDT_MULTIPLEXER_SUBJECT,
    server::{
        Server,
        ServerMetadata,
        ServerSocket,
    },
};
pub(crate) use self::{
    app_state::AppState,
    tracking::{
        track,
        track_no_ctx,
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("audit database context error: {0}")]
    AuditDatabaseContext(#[from] AuditDatabaseContextError),
    #[error("axum error: {0}")]
    Axum(#[source] hyper::Error),
    #[error("edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("error while initializing: {0}")]
    Init(#[from] init::InitError),
    #[error("kv store error: {0}")]
    KvStore(#[from] FriggError),
    #[error("nats multipler error: {0}")]
    NatsMultiplexer(#[from] ::nats_multiplexer::MultiplexerError),
    #[error("Failed to set up signal handler")]
    Signal(#[source] io::Error),
    #[error("Permissions error: {0}")]
    SpiceDb(#[from] Box<SpiceDbError>),
    #[error("unix domain socket incoming stream error: {0}")]
    Uds(#[from] uds::UdsIncomingStreamError),
}

impl From<SpiceDbError> for ServerError {
    fn from(value: SpiceDbError) -> Self {
        Box::new(value).into()
    }
}

type ServerResult<T> = std::result::Result<T, ServerError>;
