//! The rebaser [`Server`], which is the implementation for a service which processes rebases
//! between two graph structures.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use thiserror::Error;

pub mod change_set_requests;
mod config;
pub mod dvu_debouncer;
mod rebase;
mod server;

pub use config::{
    detect_and_configure_development, Config, ConfigBuilder, ConfigError, ConfigFile,
};
pub use rebaser_core::RebaserMessagingConfig;
pub use server::{Server, ServerMetadata};
pub use si_settings::{StandardConfig, StandardConfigFile};

/// An error than can be returned when a Rebaser service is running.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    /// When a Cyclone encryption key failed to be loaded
    #[error("error when loading encryption key: {0}")]
    CycloneEncryptionKey(#[source] si_crypto::CycloneEncryptionKeyError),
    /// When the DAL library fails to be initialized
    #[error("dal initialization error: {0}")]
    DalInitialization(#[from] dal::InitializationError),
    /// When failing to determine open change sets from calling DAL code
    #[error("dal open change sets error: {0}")]
    DalOpenChangeSets(#[source] si_data_pg::PgError),
    /// When a database pool error occurs
    #[error("dal pg pool error: {0}")]
    DalPgPool(#[source] Box<si_data_pg::PgPoolError>),
    /// When a DAL context fails to be created
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] dal::TransactionsError),
    /// When attempting to launch a change set task but one is already running
    #[error("existing change set task already running for id: {0}")]
    ExistingChangeSetTask(si_events::ChangeSetId),
    /// When failing to create or fetch a Jetstream consumer
    #[error("jetstream consumer error: {0}")]
    JsConsumer(#[from] si_data_nats::async_nats::jetstream::stream::ConsumerError),
    /// When failing to create a Jetstream consumer `impl Stream` of messages
    #[error("consumer stream error: {0}")]
    JsConsumerStream(#[from] si_data_nats::async_nats::jetstream::consumer::StreamError),
    /// When a LayerDb error occurs
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    /// When attempting to terminate a change set task but one is not found
    #[error("missing change set task for id: {0}")]
    MissingChangeSetTask(si_events::ChangeSetId),
    /// When a NATS client fails to be created successfully
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
    /// When a naxum-based service encounters an I/O error
    #[error("naxum error: {0}")]
    Naxum(#[source] std::io::Error),
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

type ServerResult<T> = std::result::Result<T, ServerError>;
