use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::NatsClient;
use strum::{AsRefStr, EnumString};
use telemetry::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventServer},
    layer_cache::LayerCache,
};

#[remain::sorted]
#[derive(Copy, Clone, Debug, EnumString, AsRefStr)]
#[strum(serialize_all = "snake_case")]
enum CacheName {
    Cas,
    EncryptedSecret,
    NodeWeights,
    WorkspaceSnapshots,
}

pub struct CacheUpdatesTask<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, NodeWeightValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    NodeWeightValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas_cache: LayerCache<Arc<CasValue>>,
    encrypted_secret_cache: LayerCache<Arc<EncryptedSecretValue>>,
    snapshot_cache: LayerCache<Arc<WorkspaceSnapshotValue>>,
    node_weight_cache: LayerCache<Arc<NodeWeightValue>>,
    event_channel: UnboundedReceiver<LayeredEvent>,
    shutdown_token: CancellationToken,
    tracker: TaskTracker,
}

impl<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, NodeWeightValue>
    CacheUpdatesTask<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, NodeWeightValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    NodeWeightValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    const NAME: &'static str = "LayerDB::CacheUpdatesTask";

    pub async fn create(
        instance_id: Ulid,
        nats_client: &NatsClient,
        cas_cache: LayerCache<Arc<CasValue>>,
        encrypted_secret_cache: LayerCache<Arc<EncryptedSecretValue>>,
        snapshot_cache: LayerCache<Arc<WorkspaceSnapshotValue>>,
        node_weight_cache: LayerCache<Arc<NodeWeightValue>>,
        shutdown_token: CancellationToken,
    ) -> LayerDbResult<Self> {
        let tracker = TaskTracker::new();

        let (mut layered_event_server, event_channel) =
            LayeredEventServer::create(instance_id, nats_client.clone(), shutdown_token.clone());

        tracker.spawn(async move { layered_event_server.run().await });

        Ok(Self {
            cas_cache,
            encrypted_secret_cache,
            snapshot_cache,
            node_weight_cache,
            event_channel,
            shutdown_token,
            tracker,
        })
    }

    pub async fn run(mut self) {
        let shutdown_token = self.shutdown_token.clone();
        tokio::select! {
            _ = self.process_messages() => { }
            _ = shutdown_token.cancelled() => {
            debug!(task = Self::NAME, "received cancellation");
            }
        }

        self.tracker.close();
        self.tracker.wait().await;
        debug!(task = Self::NAME, "shutdown complete");
    }

    pub async fn process_messages(&mut self) {
        while let Some(event) = self.event_channel.recv().await {
            let cache_update_task = CacheUpdateTask::new(
                self.cas_cache.clone(),
                self.encrypted_secret_cache.clone(),
                self.snapshot_cache.clone(),
            );
            self.tracker
                .spawn(async move { cache_update_task.run(event).await });
        }
    }
}

struct CacheUpdateTask<Q, R, S, T>
where
    Q: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    R: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas_cache: LayerCache<Arc<Q>>,
    encrypted_secret_cache: LayerCache<Arc<R>>,
    snapshot_cache: LayerCache<Arc<S>>,
    node_weight_cache: LayerCache<Arc<T>>,
}

impl<Q, R, S, T> CacheUpdateTask<Q, R, S, T>
where
    Q: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    R: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn new(
        cas_cache: LayerCache<Arc<Q>>,
        encrypted_secret_cache: LayerCache<Arc<R>>,
        snapshot_cache: LayerCache<Arc<S>>,
        node_weight_cache: LayerCache<Arc<T>>,
    ) -> CacheUpdateTask<Q, R, S, T> {
        CacheUpdateTask {
            cas_cache,
            encrypted_secret_cache,
            snapshot_cache,
            node_weight_cache,
        }
    }

    async fn process_message(&self, event: LayeredEvent) -> LayerDbResult<()> {
        match event.event_kind {
            crate::event::LayeredEventKind::CasInsertion => {
                if !self.cas_cache.contains(&event.key) {
                    let memory_value = self
                        .cas_cache
                        .deserialize_memory_value(&event.payload.value)?;
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.cas_cache
                        .insert_from_cache_updates(event.key, memory_value, serialized_value)
                        .await?;
                }
            }
            crate::event::LayeredEventKind::EncryptedSecretInsertion => {
                if !self.encrypted_secret_cache.contains(&event.key) {
                    let memory_value = self
                        .encrypted_secret_cache
                        .deserialize_memory_value(&event.payload.value)?;
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.encrypted_secret_cache
                        .insert_from_cache_updates(event.key, memory_value, serialized_value)
                        .await?;
                }
            }
            crate::event::LayeredEventKind::Raw => {
                warn!("Recevied a 'raw' layered event kind - this is for testing only. Bug!");
            }
            crate::event::LayeredEventKind::SnapshotWrite => {
                if !self.snapshot_cache.contains(&event.key) {
                    let memory_value = self
                        .snapshot_cache
                        .deserialize_memory_value(&event.payload.value)?;
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.snapshot_cache
                        .insert_from_cache_updates(event.key, memory_value, serialized_value)
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn run(&self, event: LayeredEvent) {
        match self.process_message(event).await {
            Ok(()) => {}
            Err(e) => {
                error!(error = %e, "error processing layerdb cache update message");
            }
        }
    }
}
