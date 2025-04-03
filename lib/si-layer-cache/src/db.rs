use change_batch::ChangeBatchDb;
use serde::Deserialize;
use si_data_pg::PgPoolConfig;
use si_runtime::DedicatedExecutor;
use std::sync::Arc;
use std::{future::IntoFuture, io};
use tokio::try_join;

use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::{NatsClient, NatsConfig};
use si_data_pg::PgPool;
use telemetry::prelude::*;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::db::encrypted_secret::EncryptedSecretDb;
use crate::db::func_run::FuncRunDb;
use crate::db::func_run_log::FuncRunLogDb;
use crate::hybrid_cache::CacheConfig;
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
pub mod change_batch;
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
    change_batch: ChangeBatchDb,
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
            pg_pool,
            nats_client,
            compute_executor,
            config.cache_config,
            token.clone(),
        )
        .await
    }

    #[instrument(name = "layer_db.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        pg_pool: PgPool,
        nats_client: NatsClient,
        compute_executor: DedicatedExecutor,
        cache_config: CacheConfig,
        token: CancellationToken,
    ) -> LayerDbResult<(Self, LayerDbGracefulShutdown)> {
        let instance_id = Ulid::new();

        let tracker = TaskTracker::new();

        let (tx, rx) = mpsc::unbounded_channel();
        let persister_client = PersisterClient::new(tx);

        let (
            cas_cache,
            change_batch_cache,
            encrypted_secret_cache,
            func_run_cache,
            func_run_log_cache,
            rebase_batch_cache,
            snapshot_cache,
        ) = try_join!(
            create_layer_cache(
                cas::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                25,
                25
            ),
            create_layer_cache(
                change_batch::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5
            ),
            create_layer_cache(
                encrypted_secret::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5
            ),
            create_layer_cache(
                func_run::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5
            ),
            create_layer_cache(
                func_run_log::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5
            ),
            create_layer_cache(
                rebase_batch::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                5,
                5
            ),
            create_layer_cache(
                workspace_snapshot::CACHE_NAME,
                pg_pool.clone(),
                cache_config.clone(),
                compute_executor.clone(),
                tracker.clone(),
                token.clone(),
                50,
                50
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
            token.clone(),
        )
        .await?;
        tracker.spawn(cache_updates_task.run());

        let persister_task = PersisterTask::create(
            rx,
            pg_pool.clone(),
            &nats_client,
            instance_id,
            token.clone(),
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
    pub cache_config: CacheConfig,
}
