use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use std::{path::Path, sync::Arc};
use telemetry::tracing::{info, warn};
use ulid::Ulid;

use crate::{
    activities::{Activity, ActivityPayloadDiscriminants, ActivityPublisher, ActivitySubscriber},
    error::LayerDbResult,
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterServer},
};
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use self::cas::{CasDb, CACHE_NAME};

mod cache_updates;
pub mod cas;

#[derive(Debug, Clone)]
pub struct LayerDb<CasValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cas: CasDb<CasValue>,
    sled: sled::Db,
    pg_pool: PgPool,
    nats_client: NatsClient,
    persister_client: PersisterClient,
    activity_publisher: ActivityPublisher,
    instance_id: Ulid,
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
}

impl<CasValue> LayerDb<CasValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn new(
        disk_path: impl AsRef<Path>,
        pg_pool: PgPool,
        nats_client: NatsClient,
    ) -> LayerDbResult<Self> {
        let instance_id = Ulid::new();

        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();

        let disk_path = disk_path.as_ref();
        let sled = sled::open(disk_path)?;

        let (tx, rx) = mpsc::unbounded_channel();
        let persister_client = PersisterClient::new(tx);

        let persister =
            PersisterServer::create(rx, sled.clone(), pg_pool.clone(), &nats_client, instance_id)
                .await?;
        let persister_cancel = cancellation_token.clone();
        tracker.spawn(async move {
            tokio::select! {
                () = persister.run() => {
                    warn!("Persister exited without being signalled");
                },
                () = persister_cancel.cancelled() => {
                    info!("Persister exiting after being cancelled");
                }
            }
        });

        let cas_cache: LayerCache<Arc<CasValue>> =
            LayerCache::new(CACHE_NAME, sled.clone(), pg_pool.clone()).await?;

        let mut cache_updates =
            cache_updates::CacheUpdates::create(instance_id, &nats_client, cas_cache.clone())
                .await?;
        let cache_update_cancel = cancellation_token.clone();
        tracker.spawn(async move {
            tokio::select! {
                () = cache_updates.run() => {
                    warn!("Cache updates exited without being signalled");
                },
                () = cache_update_cancel.cancelled() => {
                    info!("Cache updates exiting after being cancelled");
                }
            }
        });

        let cas = CasDb::new(cas_cache, persister_client.clone());

        let activity_publisher = ActivityPublisher::new(&nats_client);

        Ok(LayerDb {
            activity_publisher,
            cas,
            sled,
            pg_pool,
            persister_client,
            nats_client,
            instance_id,
            tracker,
            cancellation_token,
        })
    }

    pub async fn shutdown(&self) {
        self.tracker.close();
        self.cancellation_token.cancel();
        self.tracker.wait().await;
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
    pub fn activity_publish(&self, activity: &Activity) -> LayerDbResult<()> {
        self.activity_publisher.publish(activity)
    }

    // Subscribe to all activities, or provide an optional array of activity kinds
    // to subscribe to.
    pub async fn activity_subscribe(
        &self,
        to_receive: Option<Vec<ActivityPayloadDiscriminants>>,
    ) -> LayerDbResult<ActivitySubscriber> {
        ActivitySubscriber::new(self.instance_id, &self.nats_client, to_receive).await
    }
}
