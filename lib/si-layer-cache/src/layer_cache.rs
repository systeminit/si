use serde::{de::DeserializeOwned, Serialize};
use si_data_pg::{PgPool, PgPoolConfig};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::disk_cache::DiskCache;
use crate::error::LayerDbResult;
use crate::memory_cache::MemoryCache;
use crate::pg::PgLayer;

#[derive(Debug, Clone)]
pub struct LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    memory_cache: MemoryCache<V>,
    disk_cache: Arc<DiskCache>,
    pg: PgLayer,
}

impl<V> LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn new(name: &str, fast_disk: sled::Db, pg_pool: PgPool) -> LayerDbResult<Self> {
        let disk_cache = Arc::new(DiskCache::new(fast_disk, name.as_bytes())?);

        let pg = PgLayer::new(pg_pool, "cas");
        pg.migrate().await?;

        Ok(LayerCache {
            memory_cache: MemoryCache::new(),
            disk_cache,
            pg,
        })
    }

    async fn spawn_disk_cache_write_vec(&self, key: Arc<str>, value: Vec<u8>) -> LayerDbResult<()> {
        let self_clone = self.clone();
        let write_handle = tokio::task::spawn_blocking(move || {
            let _ = self_clone.disk_cache.insert(&key, &value);
        });
        write_handle.await?;
        Ok(())
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Option<V>> {
        Ok(match self.memory_cache.get(&key).await {
            Some(memory_value) => Some(memory_value),
            None => match self.disk_cache.get(&key)? {
                Some(value) => {
                    let deserialized: V = postcard::from_bytes(&value)?;

                    self.memory_cache.insert(key, deserialized.clone()).await;
                    Some(deserialized)
                }
                None => match self.pg.get(&key).await? {
                    Some(value) => {
                        let deserialized: V = postcard::from_bytes(&value)?;

                        self.memory_cache
                            .insert(key.clone(), deserialized.clone())
                            .await;
                        self.spawn_disk_cache_write_vec(key.clone(), value).await?;

                        Some(deserialized)
                    }
                    None => None,
                },
            },
        })
    }

    pub async fn get_bulk(&self, keys: &[Arc<str>]) -> LayerDbResult<HashMap<String, V>> {
        let mut found_keys = HashMap::new();

        for key in keys {
            if let Some(v) = self.get(key.clone()).await? {
                found_keys.insert(key.to_string(), v);
            }
        }

        Ok(found_keys)
    }

    pub fn deserialize_memory_value(&self, bytes: &[u8]) -> LayerDbResult<V> {
        postcard::from_bytes(bytes).map_err(Into::into)
    }

    pub fn memory_cache(&self) -> MemoryCache<V> {
        self.memory_cache.clone()
    }

    pub fn disk_cache(&self) -> Arc<DiskCache> {
        self.disk_cache.clone()
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
        memory_value: V,
        serialize_value: Vec<u8>,
    ) -> LayerDbResult<()> {
        self.memory_cache.insert(key.clone(), memory_value).await;
        self.spawn_disk_cache_write_vec(key.clone(), serialize_value)
            .await
    }
}

#[derive(Clone, Debug)]
pub struct LayerCacheDependencies {
    pub sled: sled::Db,
    pub pg_pool: PgPool,
}

pub async fn make_layer_cache_dependencies<P: AsRef<Path>>(
    sled_path: P,
    pg_pool_config: &PgPoolConfig,
) -> LayerDbResult<LayerCacheDependencies> {
    Ok(LayerCacheDependencies {
        sled: sled::open(sled_path)?,
        pg_pool: PgPool::new(pg_pool_config).await?,
    })
}
