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
    rebase_batch_address::RebaseBatchAddress,
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

pub const DBNAME: &str = "rebase_batches";
pub const CACHE_NAME: &str = "rebase_batches";
pub const PARTITION_KEY: &str = "rebase_batches";

#[derive(Debug, Clone)]
pub struct RebaseBatchDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub cache: Arc<LayerCache<Arc<V>>>,
    persister_client: PersisterClient,
}

impl<V> RebaseBatchDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(cache: Arc<LayerCache<Arc<V>>>, persister_client: PersisterClient) -> Self {
        Self {
            cache,
            persister_client,
        }
    }

    #[instrument(name = "rebase_batch.write", level = "debug", skip_all)]
    pub fn write(
        &self,
        value: Arc<V>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<(RebaseBatchAddress, PersisterStatusReader)> {
        let value_clone = value.clone();
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;

        let key = RebaseBatchAddress::new(&postcard_value);
        let cache_key: Arc<str> = key.to_string().into();

        self.cache.insert(cache_key.clone(), value_clone, size_hint);

        let event = LayeredEvent::new(
            LayeredEventKind::RebaseBatchWrite,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new("rebase_batches".to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok((key, reader))
    }

    #[instrument(
        name = "rebase_batch.read",
        level = "debug",
        skip_all,
        fields(
            si.rebase_batch.address = %key,
        )
    )]
    pub async fn read(&self, key: &RebaseBatchAddress) -> LayerDbResult<Option<Arc<V>>> {
        self.cache.get(key.to_string().into()).await
    }

    #[instrument(
        name = "rebase_batch.read_wait_for_memory",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.memory_cache.hit = Empty,
            si.layer_cache.memory_cache.read_wait_ms = Empty,
            si.layer_cache.memory_cache.retries = Empty,
            si.rebase_batch.address = %key,
        )
    )]
    pub async fn read_wait_for_memory(
        &self,
        key: &RebaseBatchAddress,
    ) -> LayerDbResult<Option<Arc<V>>> {
        let span = current_span_for_instrument_at!("debug");

        let key: Arc<str> = key.to_string().into();
        if let Some(batch) = self.cache.get(key.to_string().into()).await? {
            return Ok(Some(batch));
        }

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
        name = "rebase_batch.evict",
        level = "debug",
        skip_all,
        fields(
            si.rebase_batch.address = %key,
        )
    )]
    pub async fn evict(
        &self,
        key: &RebaseBatchAddress,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader> {
        let cache_key = key.to_string();
        self.cache.remove_from_memory(&cache_key);

        let event = LayeredEvent::new(
            LayeredEventKind::RebaseBatchEvict,
            Arc::new(DBNAME.to_string()),
            cache_key.into(),
            Arc::new(Vec::new()),
            Arc::new("rebase_batch".to_string()),
            None,
            tenancy,
            actor,
        );
        let reader = self.persister_client.evict_event(event)?;

        Ok(reader)
    }

    #[instrument(
        name = "rebase_batch.read_bytes_from_durable_storage",
        level = "debug",
        skip_all,
        fields(
            si.rebase_batch.address = %key,
        )
    )]
    pub async fn read_bytes_from_durable_storage(
        &self,
        key: &RebaseBatchAddress,
    ) -> LayerDbResult<Option<Vec<u8>>> {
        self.cache
            .get_bytes_from_durable_storage(key.to_string().into())
            .await
    }
}
