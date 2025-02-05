//! This module contains logic pertaining to the usage of the [`nats_multiplexer`] crate.

use std::sync::Arc;

use nats_multiplexer_client::MultiplexerClient;
use tokio::sync::Mutex;

/// The subject that the "crdt" multiplexer will be subscribing to.
pub const CRDT_MULTIPLEXER_SUBJECT: &str = "crdt.>";

/// The subject that the "ws" multiplexer will be subscribing to.
pub const WS_MULTIPLEXER_SUBJECT: &str = "si.>";

/// The subject that the "data_cache" multiplexer will be subscribing to.
pub const DATA_CACHE_MULTIPLEXER_SUBJECT: &str = "data_cache.>";

/// A grouping of multiplexer clients needed to appease the "FromRef" implementation for
/// "AppState". Yes, really.
#[derive(Debug, Clone)]
pub struct NatsMultiplexerClients {
    pub ws: Arc<Mutex<MultiplexerClient>>,
    pub crdt: Arc<Mutex<MultiplexerClient>>,
    pub data_cache: Arc<Mutex<MultiplexerClient>>,
}
