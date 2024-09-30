use std::path::PathBuf;

use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use serde::{de::DeserializeOwned, Serialize};
use si_data_pg::PgPool;
use si_runtime::DedicatedExecutor;
use telemetry::prelude::*;

use crate::db::serialize;
use crate::disk_cache::DiskCache;
use crate::error::LayerDbResult;
use crate::memory_cache::{MemoryCache, MemoryCacheConfig};
use crate::object_cache::{ObjectCache, ObjectCacheConfig};
use crate::pg::PgLayer;
use crate::LayerDbError;

#[derive(Debug, Clone)]
pub struct LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    memory_cache: MemoryCache<V>,
    disk_cache: DiskCache,
    object_cache: ObjectCache,
    pg: PgLayer,
    #[allow(dead_code)] // TODO(fnichol): remove once in use
    compute_executor: DedicatedExecutor,
}

impl<V> LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn new(
        name: &str,
        disk_path: impl Into<PathBuf>,
        object_cache_config: ObjectCacheConfig,
        pg_pool: PgPool,
        memory_cache_config: MemoryCacheConfig,
        compute_executor: DedicatedExecutor,
    ) -> LayerDbResult<Self> {
        let disk_cache = DiskCache::new(disk_path, name)?;

        let pg = PgLayer::new(pg_pool.clone(), name);

        Ok(LayerCache {
            object_cache: ObjectCache::new(object_cache_config).await?,
            memory_cache: MemoryCache::new(memory_cache_config),
            disk_cache,
            pg,
            compute_executor,
        })
    }

    async fn spawn_disk_cache_write_vec(&self, key: Arc<str>, value: Vec<u8>) -> LayerDbResult<()> {
        self.disk_cache().insert(key, value).await?;
        Ok(())
    }

    async fn spawn_object_cache_write_vec(
        &self,
        key: Arc<str>,
        value: Vec<u8>,
    ) -> LayerDbResult<()> {
        self.object_cache().insert(key, value).await?;
        Ok(())
    }

    #[instrument(
        name = "layer_cache.get",
        level = "info",
        skip_all,
        fields(
            si.layer_cache.key = key.as_ref(),
            si.layer_cache.layer.hit = Empty,
        ),
    )]
    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Option<V>> {
        let span = current_span_for_instrument_at!("debug");

        async fn cache_hit<V>(
            memory_cache: &MemoryCache<V>,
            key: Arc<str>,
            value: Vec<u8>,
            layer: &str,
            span: &Span,
        ) -> LayerDbResult<Option<V>>
        where
            V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
        {
            let deserialized: V = serialize::from_bytes(&value)?;
            memory_cache.insert(key.clone(), deserialized.clone()).await;
            span.record("si.layer_cache.layer.hit", layer);
            Ok(Some(deserialized))
        }

        if let Some(memory_value) = self.memory_cache.get(&key).await {
            return Ok(Some(memory_value));
        }

        if let Ok(value) = self.disk_cache.get(key.clone()).await {
            return cache_hit(&self.memory_cache, key, value, "disk", &span).await;
        }

        if let Some(value) = self.object_cache.get(key.clone()).await? {
            self.spawn_disk_cache_write_vec(key.clone(), value.clone())
                .await?;
            return cache_hit(&self.memory_cache, key, value, "object", &span).await;
        }

        if let Some(value) = self.pg.get(&key).await? {
            self.spawn_disk_cache_write_vec(key.clone(), value.clone())
                .await?;
            self.spawn_object_cache_write_vec(key.clone(), value.clone())
                .await?;
            return cache_hit(&self.memory_cache, key, value, "pg", &span).await;
        }

        Ok(None)
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
        if let Ok(bytes) = self.disk_cache.get(key.clone()).await {
            return Ok(Some(bytes));
        }

        if let Some(bytes) = self.object_cache.get(key.clone()).await? {
            return Ok(Some(bytes));
        }

        if let Some(bytes) = self.pg.get(&key).await? {
            return Ok(Some(bytes));
        }

        Ok(None)
    }

    pub async fn get_bulk<K>(&self, keys: &[K]) -> LayerDbResult<HashMap<K, V>>
    where
        K: Clone + Display + Eq + std::hash::Hash + std::str::FromStr,
        <K as std::str::FromStr>::Err: Display,
    {
        let mut found_values: HashMap<K, V> = HashMap::new();
        let mut not_found: Vec<Arc<str>> = vec![];

        for key in keys {
            let key_str: Arc<str> = key.to_string().into();
            if let Some(found) = match self.memory_cache.get(&key_str).await {
                Some(memory_value) => Some(memory_value),
                None => match self.disk_cache.get(key_str.clone()).await {
                    Ok(value) => {
                        let deserialized: V = serialize::from_bytes(&value[..])?;

                        self.memory_cache
                            .insert(key_str.clone(), deserialized.clone())
                            .await;
                        Some(deserialized)
                    }
                    Err(_) => {
                        not_found.push(key_str.clone());
                        None
                    }
                },
            } {
                found_values.insert(key.clone(), found);
            }
        }

        if !not_found.is_empty() {
            let mut found_keys: Vec<Arc<str>> = vec![];
            if let Some(obj_found) = self.object_cache.get_many(&not_found).await? {
                for (k, v) in obj_found {
                    let deserialized: V = serialize::from_bytes(&v)?;
                    self.memory_cache
                        .insert(k.clone().into(), deserialized.clone())
                        .await;
                    self.spawn_disk_cache_write_vec(k.clone().into(), v.clone())
                        .await?;
                    found_values.insert(
                        K::from_str(&k).map_err(|err| {
                            LayerDbError::CouldNotConvertToKeyFromString(err.to_string())
                        })?,
                        deserialized,
                    );
                    found_keys.push(k.into());
                }
            }
            // we don't need to hit the database if they are in S3
            not_found.retain(|item| !found_keys.contains(item));

            if let Some(pg_found) = self.pg.get_many(&not_found).await? {
                for (k, v) in pg_found {
                    let deserialized: V = serialize::from_bytes(&v)?;
                    self.memory_cache
                        .insert(k.clone().into(), deserialized.clone())
                        .await;
                    self.spawn_disk_cache_write_vec(k.clone().into(), v.clone())
                        .await?;
                    self.spawn_object_cache_write_vec(k.clone().into(), v)
                        .await?;
                    found_values.insert(
                        K::from_str(&k).map_err(|err| {
                            LayerDbError::CouldNotConvertToKeyFromString(err.to_string())
                        })?,
                        deserialized,
                    );
                }
            }
        }

        Ok(found_values)
    }

    pub async fn deserialize_memory_value(&self, bytes: Arc<Vec<u8>>) -> LayerDbResult<V> {
        serialize::from_bytes_async(&bytes)
            .await
            .map_err(Into::into)
    }

    pub fn memory_cache(&self) -> MemoryCache<V> {
        self.memory_cache.clone()
    }

    pub fn disk_cache(&self) -> &DiskCache {
        &self.disk_cache
    }

    pub fn object_cache(&self) -> ObjectCache {
        self.object_cache.clone()
    }

    pub fn pg(&self) -> PgLayer {
        self.pg.clone()
    }

    pub async fn remove_from_memory(&self, key: &str) {
        self.memory_cache.remove(key).await;
    }

    pub fn contains(&self, key: &str) -> bool {
        self.memory_cache.contains(key)
    }

    pub async fn insert(&self, key: Arc<str>, value: V) {
        if !self.memory_cache.contains(&key) {
            self.memory_cache.insert(key, value).await;
        }
    }

    pub async fn insert_from_cache_updates(
        &self,
        key: Arc<str>,
        serialize_value: Vec<u8>,
    ) -> LayerDbResult<()> {
        self.memory_cache
            .insert_raw_bytes(key.clone(), serialize_value.clone())
            .await;
        self.spawn_disk_cache_write_vec(key.clone(), serialize_value)
            .await
    }

    pub async fn insert_or_update(&self, key: Arc<str>, value: V) {
        self.memory_cache.insert(key, value).await;
    }

    pub async fn insert_or_update_from_cache_updates(
        &self,
        key: Arc<str>,
        serialize_value: Vec<u8>,
    ) -> LayerDbResult<()> {
        self.insert_from_cache_updates(key, serialize_value).await
    }

    pub async fn evict_from_cache_updates(&self, key: Arc<str>) -> LayerDbResult<()> {
        self.memory_cache().remove(&key).await;
        self.disk_cache().remove(key).await?;
        Ok(())
    }
}
