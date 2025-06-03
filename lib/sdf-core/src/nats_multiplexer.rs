//! This module contains logic pertaining to the usage of the [`nats_multiplexer`] crate.
use std::{
    error,
    sync::Arc,
};

use dal::WorkspacePk;
use edda_core::nats;
use nats_multiplexer_client::MultiplexerClient;
use si_data_nats::Message;
use tokio::sync::{
    Mutex,
    broadcast,
};

/// The subject that the "crdt" multiplexer will be subscribing to.
pub const CRDT_MULTIPLEXER_SUBJECT: &str = "crdt.>";

/// The subject that the "ws" multiplexer will be subscribing to.
pub const WS_MULTIPLEXER_SUBJECT: &str = "si.>";

/// A grouping of multiplexer clients needed to appease the "FromRef" implementation for
/// "AppState". Yes, really.
#[derive(Debug, Clone)]
pub struct NatsMultiplexerClients {
    pub ws: Arc<Mutex<MultiplexerClient>>,
    pub crdt: Arc<Mutex<MultiplexerClient>>,
    pub edda_updates: EddaUpdatesMultiplexerClient,
}

#[derive(Clone, Debug)]
pub struct EddaUpdatesMultiplexerClient {
    inner: Arc<Mutex<MultiplexerClient>>,
}

impl EddaUpdatesMultiplexerClient {
    pub fn new(inner: MultiplexerClient) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub async fn messages_for_workspace(
        &self,
        prefix: Option<&str>,
        workspace_id: WorkspacePk,
    ) -> Result<broadcast::Receiver<Message>, Box<dyn error::Error>> {
        let mut id_buf = WorkspacePk::array_to_str_buf();

        let subject = nats::subject::all_updates_for_workspace(
            prefix,
            workspace_id.array_to_str(&mut id_buf),
        );

        self.inner
            .try_lock()?
            .receiver(subject)
            .await
            .map_err(Into::into)
    }
}
