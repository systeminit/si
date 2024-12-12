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

mod app;
mod app_state;
mod config;
pub mod dal_wrapper;
pub mod error;
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

pub use self::{
    app::AxumApp,
    app_state::ApplicationRuntimeMode,
    config::{
        Config, ConfigBuilder, ConfigError, ConfigFile, IncomingStream, MigrationMode,
        StandardConfig, StandardConfigFile, WorkspacePermissions, WorkspacePermissionsMode,
    },
    error::ServerError,
    garbage_collection::SnapshotGarbageCollector,
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
