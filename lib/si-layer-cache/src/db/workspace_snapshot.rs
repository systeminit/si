use std::{
    sync::Arc,
    time::Instant,
};

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use si_events::{
    Actor,
    Tenancy,
    WebEvent,
    WorkspaceSnapshotAddress,
};
use telemetry::prelude::*;

use super::serialize;
use crate::{
    error::LayerDbResult,
    event::{
        LayeredEvent,
        LayeredEventKind,
    },
    layer_cache::LayerCache,
    persister::{
        PersisterClient,
        PersisterStatusReader,
    },
};

pub const DBNAME: &str = "workspace_snapshots";
pub const CACHE_NAME: &str = "workspace_snapshots";
pub const PARTITION_KEY: &str = "workspace_snapshots";

#[derive(Debug, Clone)]
pub struct WorkspaceSnapshotDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub cache: Arc<LayerCache<Arc<V>>>,
    persister_client: PersisterClient,
}

impl<V> WorkspaceSnapshotDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(cache: Arc<LayerCache<Arc<V>>>, persister_client: PersisterClient) -> Self {
        Self {
            cache,
            persister_client,
        }
    }

    #[instrument(level = "debug", skip_all,fields(
        si.layer_cache.workspace_snapshot.write_serialize = Empty,
    ))]
    pub fn write(
        &self,
        value: Arc<V>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<(WorkspaceSnapshotAddress, PersisterStatusReader)> {
        let span = Span::current();
        let write_wait = Instant::now();
        let value_clone = value.clone();
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;

        span.record(
            "si.layer_cache.workspace_snapshot.write_serialize",
            write_wait.elapsed().as_millis(),
        );

        let key = WorkspaceSnapshotAddress::new(&postcard_value);
        let cache_key: Arc<str> = key.to_string().into();

        self.cache.insert(cache_key.clone(), value_clone, size_hint);

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
        let span = current_span_for_instrument_at!("debug");

        let key: Arc<str> = key.to_string().into();
        const MAX_TRIES: i32 = 2000;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1));
        let mut tried = 0;
        let read_wait = Instant::now();
        while tried < MAX_TRIES {
            if let Some(v) = self.cache.cache().get_from_memory(key.clone()).await {
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
    pub fn evict(
        &self,
        key: &WorkspaceSnapshotAddress,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader> {
        let cache_key = key.to_string();
        self.cache.remove_from_memory(&cache_key);

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

    #[instrument(
        name = "workspace_snapshot.evict_memory_only",
        level = "debug",
        skip_all,
        fields(
            si.workspace_snapshot.address = %key,
        )
    )]
    pub fn evict_memory_only(
        &self,
        key: &WorkspaceSnapshotAddress,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader> {
        let cache_key = key.to_string();
        self.cache.remove_from_memory(&cache_key);

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
        let reader = self.persister_client.evict_memory_only_event(event)?;

        Ok(reader)
    }

    /// Used for when we want to get the exact bytes we're storing for this
    /// snapshot, useful when converting an out of date snapshot into a new one
    #[instrument(
        name = "workspace_snapshot.read_bytes_from_durable_storage",
        level = "debug",
        skip_all,
        fields(
            si.workspace_snapshot.address = %key,
        )
    )]
    pub async fn read_bytes_from_durable_storage(
        &self,
        key: &WorkspaceSnapshotAddress,
    ) -> LayerDbResult<Option<Vec<u8>>> {
        self.cache
            .get_bytes_from_durable_storage(key.to_string().into())
            .await
    }

    #[instrument(
        name = "workspace_snapshot.write_bytes_to_durable_storage",
        level = "debug",
        skip_all,
        fields(
            si.workspace_snapshot.address = %key,
        )
    )]
    pub async fn write_bytes_to_durable_storage(
        &self,
        key: &WorkspaceSnapshotAddress,
        bytes: &[u8],
    ) -> LayerDbResult<()> {
        let key = key.to_string();
        self.cache
            .pg()
            .insert(&key, "workspace_snapshot", bytes)
            .await?;

        Ok(())
    }
}
