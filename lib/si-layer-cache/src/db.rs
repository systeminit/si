use std::{
    collections::HashMap,
    future::IntoFuture,
    io,
    sync::Arc,
};

use change_batch::ChangeBatchDb;
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use si_data_nats::{
    NatsClient,
    NatsConfig,
};
use si_data_pg::{
    PgPool,
    PgPoolConfig,
};
use si_runtime::DedicatedExecutor;
use split_snapshot_rebase_batch::SplitSnapshotRebaseBatchDb;
use split_snapshot_subgraph::SplitSnapshotSubGraphDb;
use split_snapshot_supergraph::SplitSnapshotSuperGraphDb;
use telemetry::prelude::*;
use tokio::{
    sync::mpsc,
    try_join,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use ulid::Ulid;

use self::{
    cache_updates::CacheUpdatesTask,
    cas::CasDb,
    rebase_batch::RebaseBatchDb,
    workspace_snapshot::WorkspaceSnapshotDb,
};
use crate::{
    activity_client::ActivityClient,
    db::{
        encrypted_secret::EncryptedSecretDb,
        func_run::FuncRunDb,
        func_run_log::FuncRunLogDb,
    },
    error::LayerDbResult,
    hybrid_cache::CacheConfig,
    layer_cache::LayerCache,
    persister::{
        PersisterClient,
        PersisterMode,
        PersisterTask,
    },
    s3::S3Layer,
};

mod cache_updates;
pub mod cas;
pub mod change_batch;
pub mod encrypted_secret;
pub mod func_run;
pub mod func_run_log;
pub mod rebase_batch;
pub mod serialize;
pub mod split_snapshot_rebase_batch;
pub mod split_snapshot_subgraph;
pub mod split_snapshot_supergraph;
pub mod workspace_snapshot;

fn validate_config(config: &LayerDbConfig) -> LayerDbResult<()> {
    // Validate that S3 is configured when mode requires it
    if config.persister_mode != PersisterMode::PostgresOnly {
        // Config validation happens during S3Layer creation
        // If mode != PostgresOnly, we'll create S3Layers
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct LayerDb<
    CasValue,
    EncryptedSecretValue,
    WorkspaceSnapshotValue,
    RebaseBatchValue,
    SplitSnapshotSubGraphValue,
    SplitSnapshotSuperGraphValue,
    SplitSnapshotRebaseBatchValue,
> where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    RebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSnapshotSubGraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSnapshotSuperGraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSnapshotRebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas: CasDb<CasValue>,
    change_batch: ChangeBatchDb,
    encrypted_secret: EncryptedSecretDb<EncryptedSecretValue>,
    func_run: FuncRunDb,
    func_run_log: FuncRunLogDb,
    rebase_batch: RebaseBatchDb<RebaseBatchValue>,
    workspace_snapshot: WorkspaceSnapshotDb<WorkspaceSnapshotValue>,
    split_snapshot_subgraph: SplitSnapshotSubGraphDb<SplitSnapshotSubGraphValue>,
    split_snapshot_supergraph: SplitSnapshotSuperGraphDb<SplitSnapshotSuperGraphValue>,
    split_snapshot_rebase_batch: SplitSnapshotRebaseBatchDb<SplitSnapshotRebaseBatchValue>,
    pg_pool: PgPool,
    nats_client: NatsClient,
    persister_client: PersisterClient,
    activity: ActivityClient,
    instance_id: Ulid,
}

impl<
    CasValue,
    EncryptedSecretValue,
    WorkspaceSnapshotValue,
    RebaseBatchValue,
    SplitSnapshotSubGraphValue,
    SplitSnapshotSuperGraphValue,
    SplitSnapshotRebaseBatchValue,
>
    LayerDb<
        CasValue,
        EncryptedSecretValue,
        WorkspaceSnapshotValue,
        RebaseBatchValue,
        SplitSnapshotSubGraphValue,
        SplitSnapshotSuperGraphValue,
        SplitSnapshotRebaseBatchValue,
    >
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    RebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSnapshotSubGraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSnapshotSuperGraphValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    SplitSnapshotRebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    #[instrument(name = "layer_db.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: LayerDbConfig,
        compute_executor: DedicatedExecutor,
        token: CancellationToken,
    ) -> LayerDbResult<(Self, LayerDbGracefulShutdown)> {
        let pg_pool = PgPool::new(&config.pg_pool_config).await?;
        let nats_client = NatsClient::new(&config.nats_config).await?;

        Self::from_services(
            config,
            pg_pool,
            nats_client,
            compute_executor,
            token.clone(),
        )
        .await
    }

    #[instrument(name = "layer_db.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        config: LayerDbConfig,
        pg_pool: PgPool,
        nats_client: NatsClient,
        compute_executor: DedicatedExecutor,
        token: CancellationToken,
    ) -> LayerDbResult<(Self, LayerDbGracefulShutdown)> {
        let instance_id = Ulid::new();

        let tracker = TaskTracker::new();

        let (tx, rx) = mpsc::unbounded_channel();
        let persister_client = PersisterClient::new(tx);

        // Validate configuration
        validate_config(&config)?;

        // Always create S3 layers for health check validation
        use telemetry::prelude::*;

        use crate::s3::KeyTransformStrategy;

        let mut layers = HashMap::new();

        // Define cache configurations with their strategies
        let cache_configs = [
            (cas::CACHE_NAME, KeyTransformStrategy::Passthrough),
            (change_batch::CACHE_NAME, KeyTransformStrategy::Passthrough),
            (
                workspace_snapshot::CACHE_NAME,
                KeyTransformStrategy::Passthrough,
            ),
            (rebase_batch::CACHE_NAME, KeyTransformStrategy::Passthrough),
            (
                encrypted_secret::CACHE_NAME,
                KeyTransformStrategy::Passthrough,
            ),
            (func_run::CACHE_NAME, KeyTransformStrategy::ReverseKey), // ULID-based
            (func_run_log::CACHE_NAME, KeyTransformStrategy::ReverseKey), // ULID-based
            (
                split_snapshot_subgraph::CACHE_NAME,
                KeyTransformStrategy::Passthrough,
            ),
            (
                split_snapshot_supergraph::CACHE_NAME,
                KeyTransformStrategy::Passthrough,
            ),
            (
                split_snapshot_rebase_batch::CACHE_NAME,
                KeyTransformStrategy::Passthrough,
            ),
        ];

        for (cache_name, strategy) in cache_configs {
            let cache_config = config.object_storage_config.for_cache(cache_name);
            // Use cache disk path as base for S3 write queue, same as retry queue
            let queue_base_path = config.cache_config.disk_path();
            let rate_limit_config = config.object_storage_config.rate_limit.clone();
            let read_retry_config = config.object_storage_config.read_retry.clone();
            let s3_layer = S3Layer::new(
                cache_config,
                cache_name,
                strategy,
                rate_limit_config,
                read_retry_config,
                queue_base_path,
            )
            .await?;
            layers.insert(cache_name, s3_layer);
        }

        // Health check: validate S3 connectivity for ALL modes
        for (cache_name, s3_layer) in layers.iter() {
            match s3_layer.migrate().await {
                Ok(_) => {
                    info!(cache_name = cache_name, "S3 connectivity validated");
                }
                Err(e) => {
                    // Extract error details for structured logging
                    let error_kind = match &e {
                        crate::LayerDbError::S3(s3_err) => s3_err.kind(),
                        _ => "unknown",
                    };

                    warn!(
                        error = ?e,
                        error_kind = error_kind,
                        cache_name = cache_name,
                        mode = ?config.persister_mode,
                        "S3 connectivity check failed"
                    );
                }
            }
        }

        // Mode-specific layer retention
        let s3_layers = if config.persister_mode == PersisterMode::PostgresOnly {
            // Drop layers - we just validated config, don't need them for operations
            None
        } else {
            // Keep validated layers for S3 operations
            Some(Arc::new(layers))
        };

        let cache_config = config.cache_config.clone();

        let (
            cas_cache,
            change_batch_cache,
            encrypted_secret_cache,
            func_run_cache,
            func_run_log_cache,
            rebase_batch_cache,
            snapshot_cache,
            split_snapshot_subgraph_cache,
            split_snapshot_supergraph_cache,
            split_snapshot_rebase_batch_cache,
        ) = try_join!(
            create_layer_cache(
                cas::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                24,
                24,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                change_batch::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                encrypted_secret::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                func_run::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                4,
                4,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                func_run_log::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                4,
                4,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                rebase_batch::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                workspace_snapshot::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                50,
                50,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                split_snapshot_subgraph::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                1,
                1,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                split_snapshot_supergraph::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                1,
                1,
                s3_layers.clone(),
                config.persister_mode,
            ),
            create_layer_cache(
                split_snapshot_rebase_batch::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                1,
                1,
                s3_layers.clone(),
                config.persister_mode,
            )
        )?;

        let cache_updates_task = CacheUpdatesTask::create(
            instance_id,
            &nats_client,
            cas_cache.clone(),
            change_batch_cache.clone(),
            encrypted_secret_cache.clone(),
            func_run_cache.clone(),
            func_run_log_cache.clone(),
            rebase_batch_cache.clone(),
            snapshot_cache.clone(),
            split_snapshot_subgraph_cache.clone(),
            split_snapshot_supergraph_cache.clone(),
            split_snapshot_rebase_batch_cache.clone(),
            token.clone(),
        )
        .await?;
        tracker.spawn(cache_updates_task.run());

        let persister_task = PersisterTask::create(
            rx,
            pg_pool.clone(),
            &nats_client,
            instance_id,
            cache_config.disk_path().to_path_buf(), // Use cache disk path as base
            token.clone(),
            s3_layers.clone(),
            config.persister_mode,
        )
        .await?;
        tracker.spawn(persister_task.run());

        let cas = CasDb::new(cas_cache, persister_client.clone());
        let change_batch = ChangeBatchDb::new(change_batch_cache, persister_client.clone());
        let encrypted_secret =
            EncryptedSecretDb::new(encrypted_secret_cache, persister_client.clone());
        let func_run = FuncRunDb::new(func_run_cache, persister_client.clone());
        let func_run_log = FuncRunLogDb::new(func_run_log_cache, persister_client.clone());
        let workspace_snapshot = WorkspaceSnapshotDb::new(snapshot_cache, persister_client.clone());
        let rebase_batch = RebaseBatchDb::new(rebase_batch_cache, persister_client.clone());
        let split_snapshot_subgraph =
            SplitSnapshotSubGraphDb::new(split_snapshot_subgraph_cache, persister_client.clone());
        let split_snapshot_supergraph = SplitSnapshotSuperGraphDb::new(
            split_snapshot_supergraph_cache,
            persister_client.clone(),
        );
        let split_snapshot_rebase_batch = SplitSnapshotRebaseBatchDb::new(
            split_snapshot_rebase_batch_cache,
            persister_client.clone(),
        );

        let activity = ActivityClient::new(instance_id, nats_client.clone(), token.clone());
        let graceful_shutdown = LayerDbGracefulShutdown { tracker, token };

        let layerdb = LayerDb {
            activity,
            cas,
            change_batch,
            encrypted_secret,
            func_run,
            func_run_log,
            workspace_snapshot,
            pg_pool,
            persister_client,
            nats_client,
            instance_id,
            rebase_batch,
            split_snapshot_subgraph,
            split_snapshot_supergraph,
            split_snapshot_rebase_batch,
        };

        Ok((layerdb, graceful_shutdown))
    }

    pub fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
    }

    pub fn nats_client(&self) -> &NatsClient {
        &self.nats_client
    }

    pub fn persister_client(&self) -> &PersisterClient {
        &self.persister_client
    }

    pub fn cas(&self) -> &CasDb<CasValue> {
        &self.cas
    }

    pub fn change_batch(&self) -> &ChangeBatchDb {
        &self.change_batch
    }

    pub fn encrypted_secret(&self) -> &EncryptedSecretDb<EncryptedSecretValue> {
        &self.encrypted_secret
    }

    pub fn func_run(&self) -> &FuncRunDb {
        &self.func_run
    }

    pub fn func_run_log(&self) -> &FuncRunLogDb {
        &self.func_run_log
    }

    pub fn rebase_batch(&self) -> &RebaseBatchDb<RebaseBatchValue> {
        &self.rebase_batch
    }

    pub fn workspace_snapshot(&self) -> &WorkspaceSnapshotDb<WorkspaceSnapshotValue> {
        &self.workspace_snapshot
    }

    pub fn split_snapshot_subgraph(&self) -> &SplitSnapshotSubGraphDb<SplitSnapshotSubGraphValue> {
        &self.split_snapshot_subgraph
    }

    pub fn split_snapshot_supergraph(
        &self,
    ) -> &SplitSnapshotSuperGraphDb<SplitSnapshotSuperGraphValue> {
        &self.split_snapshot_supergraph
    }

    pub fn split_snapshot_rebase_batch(
        &self,
    ) -> &SplitSnapshotRebaseBatchDb<SplitSnapshotRebaseBatchValue> {
        &self.split_snapshot_rebase_batch
    }

    pub fn instance_id(&self) -> Ulid {
        self.instance_id
    }

    pub fn activity(&self) -> &ActivityClient {
        &self.activity
    }

    /// Run all migrations
    pub async fn pg_migrate(&self) -> LayerDbResult<()> {
        // This will do all migrations, not just "cas" migrations. We might want
        // to think about restructuring this
        self.cas.cache.pg().migrate().await?;

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn create_layer_cache<T>(
    name: &'static str,
    pg_pool: PgPool,
    cache_config: CacheConfig,
    compute_executor: DedicatedExecutor,
    tracker: TaskTracker,
    token: CancellationToken,
    memory_percent: u8,
    disk_percent: u8,
    s3_layers: Option<Arc<HashMap<&'static str, S3Layer>>>,
    mode: PersisterMode,
) -> LayerDbResult<Arc<LayerCache<Arc<T>>>>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    LayerCache::new(
        name,
        pg_pool,
        cache_config
            .with_name(name)
            .memory_usable_max_percent(memory_percent)
            .disk_usable_max_percent(disk_percent)
            .with_path_join(name),
        compute_executor,
        tracker,
        token,
        s3_layers,
        mode,
    )
    .await
}

#[must_use = "graceful shutdown must be spawned on runtime"]
#[derive(Debug, Clone)]
pub struct LayerDbGracefulShutdown {
    tracker: TaskTracker,
    token: CancellationToken,
}

impl IntoFuture for LayerDbGracefulShutdown {
    type Output = io::Result<()>;
    type IntoFuture = private::GracefulShutdownFuture;

    fn into_future(self) -> Self::IntoFuture {
        let Self { token, tracker } = self;

        private::GracefulShutdownFuture(Box::pin(async move {
            // Wait until token is cancelled--this is our graceful shutdown signal
            token.cancelled().await;

            // Close the tracker so no further tasks are spawned
            tracker.close();
            info!("received graceful shutdown signal, waiting for tasks to shutdown");
            // Wait for all outstanding tasks to complete
            tracker.wait().await;

            Ok(())
        }))
    }
}

mod private {
    use std::{
        fmt,
        future::Future,
        io,
        pin::Pin,
        task::{
            Context,
            Poll,
        },
    };

    pub struct GracefulShutdownFuture(
        pub(super) futures::future::BoxFuture<'static, io::Result<()>>,
    );

    impl Future for GracefulShutdownFuture {
        type Output = io::Result<()>;

        #[inline]
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.0.as_mut().poll(cx)
        }
    }

    impl fmt::Debug for GracefulShutdownFuture {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("ShutdownFuture").finish_non_exhaustive()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LayerDbConfig {
    pub pg_pool_config: PgPoolConfig,
    pub nats_config: NatsConfig,
    pub cache_config: CacheConfig,
    pub object_storage_config: crate::s3::ObjectStorageConfig,
    pub persister_mode: PersisterMode,
}
