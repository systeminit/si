use serde::Deserialize;
use si_data_pg::PgPoolConfig;
use si_runtime::DedicatedExecutor;
use std::collections::HashMap;
use std::{future::IntoFuture, io, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::{NatsClient, NatsConfig};
use si_data_pg::PgPool;
use si_events::{FuncRun, FuncRunLog};
use telemetry::prelude::*;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::db::encrypted_secret::EncryptedSecretDb;
use crate::db::func_run::FuncRunDb;
use crate::db::func_run_log::FuncRunLogDb;
use crate::disk_cache::{DiskCache, DiskCacheConfig};
use crate::memory_cache::MemoryCacheConfig;
use crate::{
    activity_client::ActivityClient,
    error::LayerDbResult,
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterTask},
};

use self::{
    cache_updates::CacheUpdatesTask, cas::CasDb, rebase_batch::RebaseBatchDb,
    workspace_snapshot::WorkspaceSnapshotDb,
};

mod cache_updates;
pub mod cas;
pub mod encrypted_secret;
pub mod func_run;
pub mod func_run_log;
pub mod rebase_batch;
pub mod serialize;
pub mod workspace_snapshot;

#[derive(Debug, Clone)]
pub struct LayerDb<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, RebaseBatchValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    RebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas: CasDb<CasValue>,
    encrypted_secret: EncryptedSecretDb<EncryptedSecretValue>,
    func_run: FuncRunDb,
    func_run_log: FuncRunLogDb,
    rebase_batch: RebaseBatchDb<RebaseBatchValue>,
    workspace_snapshot: WorkspaceSnapshotDb<WorkspaceSnapshotValue>,
    pg_pool: PgPool,
    nats_client: NatsClient,
    persister_client: PersisterClient,
    activity: ActivityClient,
    instance_id: Ulid,
}

impl<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, RebaseBatchValue>
    LayerDb<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, RebaseBatchValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    RebaseBatchValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
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
            config.disk_cache_config,
            pg_pool,
            nats_client,
            compute_executor,
            config.memory_cache_config,
            token,
        )
        .await
    }

    #[allow(unused)]
    fn spawn_cleanup_tasks(caches: &[&DiskCache], cancellation_token: CancellationToken) {
        let mut cleanup_tracker = HashMap::new();

        for disk_cache in caches {
            let write_path = disk_cache.write_path().display().to_string();
            cleanup_tracker.insert(write_path, *disk_cache);
        }

        for cache in cleanup_tracker.into_values() {
            cache.start_cleanup_task(cancellation_token.clone())
        }
    }

    #[instrument(name = "layer_db.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        disk_cache_config: DiskCacheConfig,
        pg_pool: PgPool,
        nats_client: NatsClient,
        compute_executor: DedicatedExecutor,
        memory_cache_config: MemoryCacheConfig,
        token: CancellationToken,
    ) -> LayerDbResult<(Self, LayerDbGracefulShutdown)> {
        let instance_id = Ulid::new();

        let tracker = TaskTracker::new();

        let (tx, rx) = mpsc::unbounded_channel();
        let persister_client = PersisterClient::new(tx);

        let cas_cache: LayerCache<Arc<CasValue>> = LayerCache::new(
            cas::CACHE_NAME,
            pg_pool.clone(),
            memory_cache_config.clone(),
            disk_cache_config.clone(),
            compute_executor.clone(),
        )?;

        let encrypted_secret_cache: LayerCache<Arc<EncryptedSecretValue>> = LayerCache::new(
            encrypted_secret::CACHE_NAME,
            pg_pool.clone(),
            memory_cache_config.clone(),
            disk_cache_config.clone(),
            compute_executor.clone(),
        )?;

        let func_run_cache: LayerCache<Arc<FuncRun>> = LayerCache::new(
            func_run::CACHE_NAME,
            pg_pool.clone(),
            memory_cache_config.clone(),
            disk_cache_config.clone(),
            compute_executor.clone(),
        )?;

        let func_run_log_cache: LayerCache<Arc<FuncRunLog>> = LayerCache::new(
            func_run_log::CACHE_NAME,
            pg_pool.clone(),
            memory_cache_config.clone(),
            disk_cache_config.clone(),
            compute_executor.clone(),
        )?;

        let rebase_batch_cache: LayerCache<Arc<RebaseBatchValue>> = LayerCache::new(
            rebase_batch::CACHE_NAME,
            pg_pool.clone(),
            memory_cache_config.clone(),
            disk_cache_config.clone(),
            compute_executor.clone(),
        )?;

        let snapshot_cache: LayerCache<Arc<WorkspaceSnapshotValue>> = LayerCache::new(
            workspace_snapshot::CACHE_NAME,
            pg_pool.clone(),
            memory_cache_config.clone(),
            disk_cache_config.clone(),
            compute_executor.clone(),
        )?;

        Self::spawn_cleanup_tasks(
            &[
                cas_cache.disk_cache(),
                encrypted_secret_cache.disk_cache(),
                func_run_cache.disk_cache(),
                func_run_log_cache.disk_cache(),
                rebase_batch_cache.disk_cache(),
                snapshot_cache.disk_cache(),
            ],
            token.clone(),
        );

        let cache_updates_task = CacheUpdatesTask::create(
            instance_id,
            &nats_client,
            cas_cache.clone(),
            encrypted_secret_cache.clone(),
            func_run_cache.clone(),
            func_run_log_cache.clone(),
            rebase_batch_cache.clone(),
            snapshot_cache.clone(),
            token.clone(),
        )
        .await?;
        tracker.spawn(cache_updates_task.run());

        let persister_task = PersisterTask::create(
            rx,
            disk_cache_config.clone(),
            pg_pool.clone(),
            &nats_client,
            instance_id,
            token.clone(),
        )
        .await?;
        tracker.spawn(persister_task.run());

        let cas = CasDb::new(cas_cache, persister_client.clone());
        let encrypted_secret =
            EncryptedSecretDb::new(encrypted_secret_cache, persister_client.clone());
        let func_run = FuncRunDb::new(func_run_cache, persister_client.clone());
        let func_run_log = FuncRunLogDb::new(func_run_log_cache, persister_client.clone());
        let workspace_snapshot = WorkspaceSnapshotDb::new(snapshot_cache, persister_client.clone());
        let rebase_batch = RebaseBatchDb::new(rebase_batch_cache, persister_client.clone());

        let activity = ActivityClient::new(instance_id, nats_client.clone(), token.clone());
        let graceful_shutdown = LayerDbGracefulShutdown { tracker, token };

        let layerdb = LayerDb {
            activity,
            cas,
            encrypted_secret,
            func_run,
            func_run_log,
            workspace_snapshot,
            pg_pool,
            persister_client,
            nats_client,
            instance_id,
            rebase_batch,
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
        task::{Context, Poll},
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
    pub memory_cache_config: MemoryCacheConfig,
    pub disk_cache_config: DiskCacheConfig,
}

impl LayerDbConfig {
    pub fn default_for_service(service: &str) -> Self {
        Self {
            pg_pool_config: Default::default(),
            nats_config: Default::default(),
            memory_cache_config: Default::default(),
            disk_cache_config: DiskCacheConfig::default_for_service(service),
        }
    }
}
