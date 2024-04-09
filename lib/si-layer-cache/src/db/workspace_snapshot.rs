use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use si_events::{Actor, Tenancy, WebEvent, WorkspaceSnapshotAddress};

use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterStatusReader},
};

pub const DBNAME: &str = "workspace_snapshots";
pub const CACHE_NAME: &str = "workspace_snapshots";
pub const PARTITION_KEY: &str = "workspace_snapshots";

#[derive(Debug, Clone)]
pub struct WorkspaceSnapshotDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub cache: LayerCache<Arc<V>>,
    persister_client: PersisterClient,
}

impl<V> WorkspaceSnapshotDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(cache: LayerCache<Arc<V>>, persister_client: PersisterClient) -> Self {
        Self {
            cache,
            persister_client,
        }
    }

    pub async fn write(
        &self,
        value: Arc<V>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<(WorkspaceSnapshotAddress, PersisterStatusReader)> {
        let postcard_value = postcard::to_stdvec(&value)?;
        dbg!(postcard_value.len());
        let key = WorkspaceSnapshotAddress::new(&postcard_value);
        let cache_key: Arc<str> = key.to_string().into();

        self.cache.insert(cache_key.clone(), value.clone()).await;

        let event = LayeredEvent::new(
            LayeredEventKind::SnapshotWrite,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new("workspace_snapshot".to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok((key, reader))
    }

    pub async fn read(&self, key: &WorkspaceSnapshotAddress) -> LayerDbResult<Option<Arc<V>>> {
        self.cache.get(key.to_string().into()).await
    }

    pub async fn read_wait_for_memory(
        &self,
        key: &WorkspaceSnapshotAddress,
    ) -> LayerDbResult<Option<Arc<V>>> {
        let key: Arc<str> = key.to_string().into();
        const MAX_TRIES: i32 = 2000;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1));
        let mut tried = 0;
        while tried < MAX_TRIES {
            if let Some(v) = self.cache.memory_cache().get(&key).await {
                return Ok(Some(v));
            }
            tried += 1;
            interval.tick().await;
        }
        self.cache.get(key.to_string().into()).await
    }
}
