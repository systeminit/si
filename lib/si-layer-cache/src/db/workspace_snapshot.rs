use std::{sync::Arc, time::Instant};

use serde::{de::DeserializeOwned, Serialize};
use si_events::{Actor, Tenancy, WebEvent, WorkspaceSnapshotAddress};
use telemetry::prelude::*;

use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterStatusReader},
};

use super::serialize;

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
        let postcard_value = serialize::to_vec(&value)?;
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

    #[instrument(
        name = "workspace_snapshot.read",
        level = "debug",
        skip_all,
        fields(
            si.workspace_snapshot.address = %key,
        )
    )]
    pub async fn read(&self, key: &WorkspaceSnapshotAddress) -> LayerDbResult<Option<Arc<V>>> {
        self.cache.get(key.to_string().into()).await
    }

    #[instrument(
        name = "workspace_snapshot.read_wait_for_memory",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.memory_cache.hit = Empty,
            si.layer_cache.memory_cache.read_wait_ms = Empty,
            si.layer_cache.memory_cache.retries = Empty,
            si.workspace_snapshot.address = %key,
        )
    )]
    pub async fn read_wait_for_memory(
        &self,
        key: &WorkspaceSnapshotAddress,
    ) -> LayerDbResult<Option<Arc<V>>> {
        let span = Span::current();

        let key: Arc<str> = key.to_string().into();
        const MAX_TRIES: i32 = 2000;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1));
        let mut tried = 0;
        let read_wait = Instant::now();
        while tried < MAX_TRIES {
            if let Some(v) = self.cache.memory_cache().get(&key).await {
                span.record("si.layer_cache.memory_cache.hit", true);
                span.record(
                    "si.layer_cache.memory_cache.read_wait_ms",
                    read_wait.elapsed().as_millis(),
                );
                span.record("si.layer_cache.memory_cache.retries", tried);
                return Ok(Some(v));
            }
            tried += 1;
            interval.tick().await;
        }

        span.record("si.layer_cache.memory_cache.hit", false);
        self.cache.get(key.to_string().into()).await
    }

    #[instrument(
        name = "workspace_snapshot.evict",
        level = "debug",
        skip_all,
        fields(
            si.workspace_snapshot.address = %key,
        )
    )]
    pub async fn evict(
        &self,
        key: &WorkspaceSnapshotAddress,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader> {
        let cache_key = key.to_string();
        self.cache.remove_from_memory(&cache_key).await;

        let event = LayeredEvent::new(
            LayeredEventKind::SnapshotEvict,
            Arc::new(DBNAME.to_string()),
            cache_key.into(),
            Arc::new(Vec::new()),
            Arc::new("workspace_snapshot".to_string()),
            None,
            tenancy,
            actor,
        );
        let reader = self.persister_client.evict_event(event)?;

        Ok(reader)
    }
}
