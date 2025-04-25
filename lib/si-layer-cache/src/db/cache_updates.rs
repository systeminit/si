use std::sync::Arc;

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use si_data_nats::NatsClient;
use si_events::{
    FuncRun,
    FuncRunLog,
    change_batch::ChangeBatch,
};
use telemetry::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use ulid::Ulid;

use crate::{
    error::LayerDbResult,
    event::{
        LayeredEvent,
        LayeredEventServer,
    },
    layer_cache::LayerCache,
};

pub struct CacheUpdatesTask<
    CasValue,
    EncryptedSecretValue,
    WorkspaceSnapshotValue,
    RebaseBatchValue,
    SplitSubgraphValue,
    SplitSupergraphValue,
    SplitRebaseBatchValue,
> where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    RebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSubgraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSupergraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitRebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas_cache: Arc<LayerCache<Arc<CasValue>>>,
    change_batch_cache: Arc<LayerCache<Arc<ChangeBatch>>>,
    encrypted_secret_cache: Arc<LayerCache<Arc<EncryptedSecretValue>>>,
    func_run_cache: Arc<LayerCache<Arc<FuncRun>>>,
    func_run_log_cache: Arc<LayerCache<Arc<FuncRunLog>>>,
    rebase_batch_cache: Arc<LayerCache<Arc<RebaseBatchValue>>>,
    snapshot_cache: Arc<LayerCache<Arc<WorkspaceSnapshotValue>>>,
    split_subgraph_cache: Arc<LayerCache<Arc<SplitSubgraphValue>>>,
    split_supergraph_cache: Arc<LayerCache<Arc<SplitSupergraphValue>>>,
    split_rebase_batch_cache: Arc<LayerCache<Arc<SplitRebaseBatchValue>>>,
    event_channel: UnboundedReceiver<LayeredEvent>,
    shutdown_token: CancellationToken,
    tracker: TaskTracker,
}

impl<
    CasValue,
    EncryptedSecretValue,
    WorkspaceSnapshotValue,
    RebaseBatchValue,
    SplitSubgraphValue,
    SplitSupergraphValue,
    SplitRebaseBatchValue,
>
    CacheUpdatesTask<
        CasValue,
        EncryptedSecretValue,
        WorkspaceSnapshotValue,
        RebaseBatchValue,
        SplitSubgraphValue,
        SplitSupergraphValue,
        SplitRebaseBatchValue,
    >
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    RebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSubgraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSupergraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitRebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    const NAME: &'static str = "LayerDB::CacheUpdatesTask";

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        instance_id: Ulid,
        nats_client: &NatsClient,
        cas_cache: Arc<LayerCache<Arc<CasValue>>>,
        change_batch_cache: Arc<LayerCache<Arc<ChangeBatch>>>,
        encrypted_secret_cache: Arc<LayerCache<Arc<EncryptedSecretValue>>>,
        func_run_cache: Arc<LayerCache<Arc<FuncRun>>>,
        func_run_log_cache: Arc<LayerCache<Arc<FuncRunLog>>>,
        rebase_batch_cache: Arc<LayerCache<Arc<RebaseBatchValue>>>,
        snapshot_cache: Arc<LayerCache<Arc<WorkspaceSnapshotValue>>>,
        split_subgraph_cache: Arc<LayerCache<Arc<SplitSubgraphValue>>>,
        split_supergraph_cache: Arc<LayerCache<Arc<SplitSupergraphValue>>>,
        split_snapshot_rebase_batch_cache: Arc<LayerCache<Arc<SplitRebaseBatchValue>>>,
        shutdown_token: CancellationToken,
    ) -> LayerDbResult<Self> {
        let tracker = TaskTracker::new();

        let (mut layered_event_server, event_channel) =
            LayeredEventServer::create(instance_id, nats_client.clone(), shutdown_token.clone());

        tracker.spawn(async move { layered_event_server.run().await });

        Ok(Self {
            cas_cache,
            change_batch_cache,
            encrypted_secret_cache,
            func_run_cache,
            func_run_log_cache,
            rebase_batch_cache,
            snapshot_cache,
            split_subgraph_cache,
            split_supergraph_cache,
            split_rebase_batch_cache: split_snapshot_rebase_batch_cache,
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
                self.change_batch_cache.clone(),
                self.encrypted_secret_cache.clone(),
                self.func_run_cache.clone(),
                self.func_run_log_cache.clone(),
                self.snapshot_cache.clone(),
                self.rebase_batch_cache.clone(),
                self.split_subgraph_cache.clone(),
                self.split_supergraph_cache.clone(),
                self.split_rebase_batch_cache.clone(),
            );
            self.tracker
                .spawn(async move { cache_update_task.run(event).await });
        }
    }
}

struct CacheUpdateTask<Q, R, S, T, U, V, W>
where
    Q: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    R: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    U: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    W: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas_cache: Arc<LayerCache<Arc<Q>>>,
    change_batch_cache: Arc<LayerCache<Arc<ChangeBatch>>>,
    encrypted_secret_cache: Arc<LayerCache<Arc<R>>>,
    func_run_cache: Arc<LayerCache<Arc<FuncRun>>>,
    func_run_log_cache: Arc<LayerCache<Arc<FuncRunLog>>>,
    snapshot_cache: Arc<LayerCache<Arc<S>>>,
    rebase_batch_cache: Arc<LayerCache<Arc<T>>>,
    split_subgraph_cache: Arc<LayerCache<Arc<U>>>,
    split_supergraph_cache: Arc<LayerCache<Arc<V>>>,
    split_rebase_batch_cache: Arc<LayerCache<Arc<W>>>,
}

impl<Q, R, S, T, U, V, W> CacheUpdateTask<Q, R, S, T, U, V, W>
where
    Q: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    R: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    U: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    W: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    #[allow(clippy::too_many_arguments)]
    fn new(
        cas_cache: Arc<LayerCache<Arc<Q>>>,
        change_batch_cache: Arc<LayerCache<Arc<ChangeBatch>>>,
        encrypted_secret_cache: Arc<LayerCache<Arc<R>>>,
        func_run_cache: Arc<LayerCache<Arc<FuncRun>>>,
        func_run_log_cache: Arc<LayerCache<Arc<FuncRunLog>>>,
        snapshot_cache: Arc<LayerCache<Arc<S>>>,
        rebase_batch_cache: Arc<LayerCache<Arc<T>>>,
        split_subgraph_cache: Arc<LayerCache<Arc<U>>>,
        split_supergraph_cache: Arc<LayerCache<Arc<V>>>,
        split_rebase_batch_cache: Arc<LayerCache<Arc<W>>>,
    ) -> CacheUpdateTask<Q, R, S, T, U, V, W> {
        CacheUpdateTask {
            cas_cache,
            change_batch_cache,
            encrypted_secret_cache,
            func_run_cache,
            func_run_log_cache,
            snapshot_cache,
            rebase_batch_cache,
            split_subgraph_cache,
            split_supergraph_cache,
            split_rebase_batch_cache,
        }
    }

    async fn process_message(&self, event: LayeredEvent) -> LayerDbResult<()> {
        match event.event_kind {
            crate::event::LayeredEventKind::CasInsertion => {
                if !self.cas_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.cas_cache
                        .insert_from_cache_updates(event.key, serialized_value);
                }
            }
            crate::event::LayeredEventKind::ChangeBatchWrite => {
                if !self.change_batch_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.change_batch_cache
                        .insert_from_cache_updates(event.key, serialized_value);
                }
            }
            crate::event::LayeredEventKind::ChangeBatchEvict => {
                self.change_batch_cache.evict_from_cache_updates(event.key);
            }
            crate::event::LayeredEventKind::EncryptedSecretInsertion => {
                if !self.encrypted_secret_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.encrypted_secret_cache
                        .insert_from_cache_updates(event.key, serialized_value);
                }
            }
            crate::event::LayeredEventKind::FuncRunWrite => {
                let serialized_value =
                    Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                self.func_run_cache
                    .insert_or_update_from_cache_updates(event.key, serialized_value);
            }
            crate::event::LayeredEventKind::FuncRunLogWrite => {
                let serialized_value =
                    Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                self.func_run_log_cache
                    .insert_or_update_from_cache_updates(event.key, serialized_value);
            }
            crate::event::LayeredEventKind::Raw => {
                warn!("Recevied a 'raw' layered event kind - this is for testing only. Bug!");
            }

            crate::event::LayeredEventKind::RebaseBatchWrite => {
                if !self.rebase_batch_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.rebase_batch_cache
                        .insert_from_cache_updates(event.key, serialized_value);
                }
            }
            crate::event::LayeredEventKind::RebaseBatchEvict => {
                self.rebase_batch_cache.evict_from_cache_updates(event.key);
            }
            crate::event::LayeredEventKind::SnapshotWrite => {
                if !self.snapshot_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.snapshot_cache
                        .insert_from_cache_updates(event.key, serialized_value);
                }
            }
            crate::event::LayeredEventKind::SnapshotEvict => {
                self.snapshot_cache.evict_from_cache_updates(event.key);
            }
            crate::event::LayeredEventKind::SplitRebaseBatchEvict => {
                self.split_rebase_batch_cache
                    .evict_from_cache_updates(event.key);
            }
            crate::event::LayeredEventKind::SplitRebaseBatchWrite => {
                if !self.split_rebase_batch_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.split_rebase_batch_cache
                        .insert_from_cache_updates(event.key, serialized_value);
                }
            }
            crate::event::LayeredEventKind::SplitSnapshotSubGraphEvict => {
                self.split_subgraph_cache
                    .evict_from_cache_updates(event.key);
            }
            crate::event::LayeredEventKind::SplitSnapshotSubGraphWrite => {
                if !self.split_subgraph_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.split_subgraph_cache
                        .insert_from_cache_updates(event.key, serialized_value);
                }
            }
            crate::event::LayeredEventKind::SplitSnapshotSuperGraphEvict => {
                self.split_supergraph_cache
                    .evict_from_cache_updates(event.key);
            }
            crate::event::LayeredEventKind::SplitSnapshotSuperGraphWrite => {
                if !self.split_supergraph_cache.contains(&event.key) {
                    let serialized_value =
                        Arc::try_unwrap(event.payload.value).unwrap_or_else(|arc| (*arc).clone());
                    self.split_supergraph_cache
                        .insert_from_cache_updates(event.key, serialized_value);
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
