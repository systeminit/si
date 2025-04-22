use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use si_crypto::VeritechDecryptionKey;
use si_data_nats::NatsClient;
use si_pool_noodle::{
    PoolNoodle,
    instance::cyclone::{
        LocalUdsInstance,
        LocalUdsInstanceSpec,
    },
};
use tokio::sync::{
    Mutex,
    oneshot,
};
use veritech_core::ExecutionId;

use crate::server::ServerMetadata;

/// Application state.
#[derive(Clone, Debug)]
pub struct AppState {
    #[allow(unused)]
    pub metadata: Arc<ServerMetadata>,
    // NOTE(nick,fletcher,scott): this implements clone and the inner bits are wrapped in an Arc.
    // If that changes, then I hope you read this comment before that happens.
    pub cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    pub decryption_key: Arc<VeritechDecryptionKey>,
    // TODO(nick,fletcher,scott): make this mutable at runtime.
    pub cyclone_client_execution_timeout: Duration,
    pub nats: NatsClient,
    pub kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
}

impl AppState {
    /// Creates a new [`AppState`].
    pub fn new(
        metadata: Arc<ServerMetadata>,
        cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
        decryption_key: Arc<VeritechDecryptionKey>,
        cyclone_client_execution_timeout: Duration,
        nats: NatsClient,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
    ) -> Self {
        Self {
            metadata,
            cyclone_pool,
            decryption_key,
            cyclone_client_execution_timeout,
            nats,
            kill_senders,
        }
    }

    /// Determine if the nats client is using a subject prefix.
    pub fn nats_subject_has_prefix(&self) -> bool {
        self.nats.metadata().subject_prefix().is_some()
    }
}

/// Application state for killing function executions.
#[derive(Clone, Debug)]
pub struct KillAppState {
    #[allow(unused)]
    pub metadata: Arc<ServerMetadata>,
    pub nats: NatsClient,
    pub kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
}

impl KillAppState {
    /// Creates a new [`KillAppState`].
    pub fn new(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
    ) -> Self {
        Self {
            metadata,
            nats,
            kill_senders,
        }
    }
}
