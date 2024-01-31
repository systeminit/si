//! This module contains logic pertaining to the usage of the [`nats_multiplexer`] crate.

use nats_multiplexer_client::MultiplexerClient;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The subject that the "crdt" multiplexer will be subscribing to.
pub const CRDT_MULTIPLEXER_SUBJECT: &str = "crdt.>";

/// The subject that the "ws" multiplexer will be subscribing to.
pub const WS_MULTIPLEXER_SUBJECT: &str = "si.>";

/// A grouping of multiplexer clients needed to appease the "FromRef" implementation for "AppState". Yes, really.
#[derive(Debug, Clone)]
pub(crate) struct NatsMultiplexerClients {
    pub ws: Arc<Mutex<MultiplexerClient>>,
    pub crdt: Arc<Mutex<MultiplexerClient>>,
}
