use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    str::FromStr,
    sync::Arc,
};

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use si_data_pg::PgPool;
use si_runtime::DedicatedExecutor;
use telemetry::prelude::*;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use crate::{
    LayerDbError,
    db::serialize,
    error::LayerDbResult,
    hybrid_cache::{
        Cache,
        CacheConfig,
    },
    pg::PgLayer,
};

#[derive(Debug, Clone)]
pub struct LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cache: Cache<V>,
    name: String,
    pg: PgLayer,
    #[allow(dead_code)]
    compute_executor: DedicatedExecutor,
}

impl<V> LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn new(
        name: &str,
        pg_pool: PgPool,
        cache_config: CacheConfig,
        #[allow(dead_code)] compute_executor: DedicatedExecutor,
        tracker: TaskTracker,
        token: CancellationToken,
    ) -> LayerDbResult<Arc<Self>> {
        let cache = Cache::new(cache_config).await?;

        let pg = PgLayer::new(pg_pool.clone(), name);

        let lc: Arc<LayerCache<V>> = LayerCache {
            cache,
            name: name.to_string(),
            pg,
            compute_executor,
        }
        .into();

        tracker.spawn(lc.clone().shutdown_handler(token.clone()));
        Ok(lc)
    }

    async fn shutdown_handler(self: Arc<Self>, token: CancellationToken) -> LayerDbResult<()> {
        token.cancelled().await;
        debug!("shutting down layer cache {}", self.name);
        // foyer will wait on all outstanding flush and reclaim threads here
        self.cache().close().await?;
        Ok(())
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Option<V>> {
        Ok(match self.cache.get(key.clone()).await {
            Some(memory_value) => Some(memory_value),

            None => match self.pg.get(&key).await? {
                Some(bytes) => {
                    let deserialized: V = serialize::from_bytes(&bytes)?;

                    self.cache
                        .insert(key.clone(), deserialized.clone(), bytes.len());

                    Some(deserialized)
                }
                None => None,
            },
        })
    }

    #[instrument(
        name = "layer_cache.get_from_memory",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.key = key.as_ref(),
        ),
    )]
    pub async fn get_from_memory(&self, key: Arc<str>) -> LayerDbResult<Option<V>> {
        Ok(self.cache().get_from_memory(key).await)
    }

    #[instrument(
        name = "layer_cache.get_bytes_from_durable_storage",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.key = key.as_ref(),
        ),
    )]
    pub async fn get_bytes_from_durable_storage(
        &self,
        key: Arc<str>,
    ) -> LayerDbResult<Option<Vec<u8>>> {
        self.pg.get(&key).await
    }

    pub async fn get_bulk<K>(&self, keys: &[K]) -> LayerDbResult<HashMap<K, V>>
    where
        K: Clone + Display + Eq + Hash + FromStr,
        <K as FromStr>::Err: Display,
    {
        let mut found_keys = HashMap::new();
        let mut not_found: Vec<Arc<str>> = vec![];

        for key in keys {
            let key_str: Arc<str> = key.to_string().into();
            if let Some(found) = match self.cache.get(key_str.clone()).await {
                Some(value) => Some(value),
                None => {
                    not_found.push(key_str.clone());
                    None
                }
            } {
                found_keys.insert(key.clone(), found);
            }
        }

        if !not_found.is_empty() {
            if let Some(pg_found) = self.pg.get_many(&not_found).await? {
                for (k, bytes) in pg_found {
                    let deserialized: V = serialize::from_bytes(&bytes)?;
                    self.cache
                        .insert(k.clone().into(), deserialized.clone(), bytes.len());
                    found_keys.insert(
                        K::from_str(&k).map_err(|err| {
                            LayerDbError::CouldNotConvertToKeyFromString(err.to_string())
                        })?,
                        deserialized,
                    );
                }
            }
        }

        Ok(found_keys)
    }

    pub async fn deserialize_memory_value(&self, bytes: Arc<Vec<u8>>) -> LayerDbResult<V> {
        serialize::from_bytes_async(&bytes).await
    }

    pub fn cache(&self) -> Cache<V> {
        self.cache.clone()
    }

    pub fn pg(&self) -> PgLayer {
        self.pg.clone()
    }

    pub fn remove_from_memory(&self, key: &str) {
        self.cache.remove(key);
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains(key)
    }

    pub fn insert(&self, key: Arc<str>, value: V, size_hint: usize) {
        if !self.cache.contains(&key) {
            self.cache.insert(key, value, size_hint);
        }
    }

    pub fn insert_from_cache_updates(&self, key: Arc<str>, serialize_value: Vec<u8>) {
        self.cache
            .insert_raw_bytes(key.clone(), serialize_value.clone());
    }

    pub fn insert_or_update(&self, key: Arc<str>, value: V, size_hint: usize) {
        self.cache.insert(key, value, size_hint);
    }

    pub fn insert_or_update_from_cache_updates(&self, key: Arc<str>, serialize_value: Vec<u8>) {
        self.insert_from_cache_updates(key, serialize_value)
    }

    pub fn evict_from_cache_updates(&self, key: Arc<str>) {
        self.cache.remove(&key);
    }
}
