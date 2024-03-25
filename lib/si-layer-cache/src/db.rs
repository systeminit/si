use std::{future::IntoFuture, io, path::Path, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use telemetry::prelude::*;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::{
    activities::{
        Activity, ActivityPayloadDiscriminants, ActivityPublisher, ActivityStream,
        RebaserRequestsWorkQueueStream,
    },
    error::LayerDbResult,
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterTask},
};

use self::{cache_updates::CacheUpdatesTask, cas::CasDb, workspace_snapshot::WorkspaceSnapshotDb};

mod cache_updates;
pub mod cas;
pub mod workspace_snapshot;

#[derive(Debug, Clone)]
pub struct LayerDb<CasValue, WorkspaceSnapshotValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas: CasDb<CasValue>,
    workspace_snapshot: WorkspaceSnapshotDb<WorkspaceSnapshotValue>,
    sled: sled::Db,
    pg_pool: PgPool,
    nats_client: NatsClient,
    persister_client: PersisterClient,
    activity_publisher: ActivityPublisher,
    instance_id: Ulid,
}

impl<CasValue, WorkspaceSnapshotValue> LayerDb<CasValue, WorkspaceSnapshotValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn initialize(
        disk_path: impl AsRef<Path>,
        pg_pool: PgPool,
        nats_client: NatsClient,
        token: CancellationToken,
    ) -> LayerDbResult<(Self, LayerDbGracefulShutdown)> {
        let instance_id = Ulid::new();

        let tracker = TaskTracker::new();

        let disk_path = disk_path.as_ref();
        let sled = sled::open(disk_path)?;

        let (tx, rx) = mpsc::unbounded_channel();
        let persister_client = PersisterClient::new(tx);

        let cas_cache: LayerCache<Arc<CasValue>> =
            LayerCache::new(cas::CACHE_NAME, sled.clone(), pg_pool.clone()).await?;

        let snapshot_cache: LayerCache<Arc<WorkspaceSnapshotValue>> = LayerCache::new(
            workspace_snapshot::CACHE_NAME,
            sled.clone(),
            pg_pool.clone(),
        )
        .await?;

        let cache_updates_task = CacheUpdatesTask::create(
            instance_id,
            &nats_client,
            cas_cache.clone(),
            snapshot_cache.clone(),
            token.clone(),
        )
        .await?;
        tracker.spawn(cache_updates_task.run());

        let persister_task = PersisterTask::create(
            rx,
            sled.clone(),
            pg_pool.clone(),
            &nats_client,
            instance_id,
            token.clone(),
        )
        .await?;
        tracker.spawn(persister_task.run());

        let cas = CasDb::new(cas_cache, persister_client.clone());
        let workspace_snapshot = WorkspaceSnapshotDb::new(snapshot_cache, persister_client.clone());
        let activity_publisher = ActivityPublisher::new(&nats_client);

        let graceful_shutdown = LayerDbGracefulShutdown { tracker, token };

        let layerdb = LayerDb {
            activity_publisher,
            cas,
            workspace_snapshot,
            sled,
            pg_pool,
            persister_client,
            nats_client,
            instance_id,
        };

        Ok((layerdb, graceful_shutdown))
    }

    pub fn sled(&self) -> &sled::Db {
        &self.sled
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

    pub fn workspace_snapshot(&self) -> &WorkspaceSnapshotDb<WorkspaceSnapshotValue> {
        &self.workspace_snapshot
    }

    pub fn instance_id(&self) -> Ulid {
        self.instance_id
    }

    /// Run all migrations
    pub async fn pg_migrate(&self) -> LayerDbResult<()> {
        // This will do all migrations, not just "cas" migrations. We might want
        // to think about restructuring this
        self.cas.cache.pg().migrate().await?;

        Ok(())
    }

    // Publish an activity
    pub async fn publish_activity(&self, activity: &Activity) -> LayerDbResult<()> {
        self.activity_publisher.publish(activity).await
    }

    // Subscribe to all activities, or provide an optional array of activity kinds
    // to subscribe to.
    pub async fn subscribe_activities(
        &self,
        to_receive: impl IntoIterator<Item = ActivityPayloadDiscriminants>,
    ) -> LayerDbResult<ActivityStream> {
        ActivityStream::create(self.instance_id, &self.nats_client, Some(to_receive)).await
    }

    pub async fn subscribe_all_activities(&self) -> LayerDbResult<ActivityStream> {
        ActivityStream::create(
            self.instance_id,
            &self.nats_client,
            None::<std::vec::IntoIter<_>>,
        )
        .await
    }

    pub async fn subscribe_rebaser_requests_work_queue(
        &self,
    ) -> LayerDbResult<RebaserRequestsWorkQueueStream> {
        RebaserRequestsWorkQueueStream::create(&self.nats_client).await
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
            trace!("received graceful shutdown signal, waiting for tasks to shutdown");
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
