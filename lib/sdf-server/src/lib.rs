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

use thiserror::Error;

mod app;
mod app_state;
mod config;
mod extract;
mod init;
mod migrations;
mod nats_multiplexer;
mod routes;
mod runnable;
mod server;
pub mod service;
mod tracking;
mod uds;
pub mod util;

pub use self::{
    app::AxumApp,
    app_state::ApplicationRuntimeMode,
    config::{
        detect_and_configure_development, Config, ConfigBuilder, ConfigError, ConfigFile,
        IncomingStream, MigrationMode, StandardConfig, StandardConfigFile, WorkspacePermissions,
        WorkspacePermissionsMode,
    },
    migrations::Migrator,
    nats_multiplexer::CRDT_MULTIPLEXER_SUBJECT,
    server::{Server, ServerMetadata, ServerSocket},
};
pub(crate) use self::{
    app_state::AppState,
    tracking::{track, track_no_ctx},
};
pub use dal::{
    feature_flags::{FeatureFlag, FeatureFlagService},
    JobQueueProcessor, NatsProcessor, ServicesContext,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("axum error: {0}")]
    Axum(#[source] hyper::Error),
    #[error("error while initializing: {0}")]
    Init(#[from] init::InitError),
    #[error("nats multipler error: {0}")]
    NatsMultiplexer(#[from] ::nats_multiplexer::MultiplexerError),
    #[error("Failed to set up signal handler")]
    Signal(#[source] io::Error),
    #[error("unix domain socket incoming stream error: {0}")]
    Uds(#[from] uds::UdsIncomingStreamError),
}

type ServerResult<T> = std::result::Result<T, ServerError>;
