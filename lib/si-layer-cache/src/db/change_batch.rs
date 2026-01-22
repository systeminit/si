use std::{
    sync::Arc,
    time::Instant,
};

use si_events::{
    Actor,
    Tenancy,
    WebEvent,
    change_batch::{
        ChangeBatch,
        ChangeBatchAddress,
    },
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

pub const DBNAME: &str = "change_batches";
pub const CACHE_NAME: &str = DBNAME;
pub const PARTITION_KEY: &str = CACHE_NAME;
const SORT_KEY: &str = CACHE_NAME;

#[derive(Debug, Clone)]
pub struct ChangeBatchDb {
    pub cache: Arc<LayerCache<Arc<ChangeBatch>>>,
    persister_client: PersisterClient,
}

impl ChangeBatchDb {
    pub fn new(
        cache: Arc<LayerCache<Arc<ChangeBatch>>>,
        persister_client: PersisterClient,
    ) -> Self {
        Self {
            cache,
            persister_client,
        }
    }

    #[instrument(name = "change_batch.write", level = "debug", skip_all)]
    pub fn write(
        &self,
        value: Arc<ChangeBatch>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<(ChangeBatchAddress, PersisterStatusReader)> {
        let value_clone = value.clone();
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;

        let key = ChangeBatchAddress::new(&postcard_value);
        let cache_key: Arc<str> = key.to_string().into();

        self.cache.insert(cache_key.clone(), value_clone, size_hint);

        let event = LayeredEvent::new(
            LayeredEventKind::ChangeBatchWrite,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new(SORT_KEY.to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok((key, reader))
    }

    #[instrument(
        name = "change_batch.read",
        level = "debug",
        skip_all,
        fields(
            si.change_batch.address = %key,
        )
    )]
    pub async fn read(&self, key: &ChangeBatchAddress) -> LayerDbResult<Option<Arc<ChangeBatch>>> {
        self.cache.get(key.to_string().into()).await
    }

    #[instrument(
        name = "change_batch.read_wait_for_memory",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.memory_cache.hit = Empty,
            si.layer_cache.memory_cache.read_wait_ms = Empty,
            si.layer_cache.memory_cache.retries = Empty,
            si.change_batch.address = %key,
        )
    )]
    pub async fn read_wait_for_memory(
        &self,
        key: &ChangeBatchAddress,
    ) -> LayerDbResult<Option<Arc<ChangeBatch>>> {
        let span = current_span_for_instrument_at!("debug");

        if let Some(batch) = self.cache.get(key.to_string().into()).await? {
            return Ok(Some(batch));
        }

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
        name = "change_batch.evict",
        level = "debug",
        skip_all,
        fields(
            si.change_batch.address = %key,
        )
    )]
    pub async fn evict(
        &self,
        key: &ChangeBatchAddress,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader> {
        let cache_key = key.to_string();
        self.cache.remove_from_memory(&cache_key);

        let event = LayeredEvent::new(
            LayeredEventKind::ChangeBatchEvict,
            Arc::new(DBNAME.to_string()),
            cache_key.into(),
            Arc::new(Vec::new()),
            Arc::new(SORT_KEY.to_string()),
            None,
            tenancy,
            actor,
        );
        let reader = self.persister_client.evict_event(event)?;

        Ok(reader)
    }

    #[instrument(
        name = "change_batch.read_bytes_from_durable_storage",
        level = "debug",
        skip_all,
        fields(
            si.change_batch.address = %key,
        )
    )]
    pub async fn read_bytes_from_durable_storage(
        &self,
        key: &ChangeBatchAddress,
    ) -> LayerDbResult<Option<Vec<u8>>> {
        self.cache
            .get_bytes_from_durable_storage(key.to_string().into())
            .await
    }
}
