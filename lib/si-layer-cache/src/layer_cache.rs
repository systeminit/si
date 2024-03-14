use serde::{de::DeserializeOwned, Serialize};
use si_data_pg::{PgPool, PgPoolConfig};
use std::collections::HashMap;
use std::{hash::Hash, path::Path, sync::Arc};

use crate::disk_cache::DiskCache;
use crate::error::LayerDbResult;
use crate::memory_cache::MemoryCache;
use crate::pg::PgLayer;

#[derive(Debug, Clone)]
pub struct LayerCache<K, V>
where
    K: AsRef<[u8]> + Eq + Hash + Copy + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    memory_cache: MemoryCache<K, V>,
    disk_cache: Arc<DiskCache<K>>,
    pg: PgLayer<K>,
}

impl<K, V> LayerCache<K, V>
where
    K: AsRef<[u8]> + Eq + Hash + Copy + Send + Sync + 'static,
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

    async fn spawn_disk_cache_write_vec(&self, key: K, value: Vec<u8>) -> LayerDbResult<()> {
        let self_clone = self.clone();
        let write_handle = tokio::task::spawn_blocking(move || {
            let _ = self_clone.disk_cache.insert(key, &value);
        });
        write_handle.await?;
        Ok(())
    }

    pub async fn get(&self, key: &K) -> LayerDbResult<Option<V>> {
        Ok(match self.memory_cache.get(key).await {
            Some(memory_value) => Some(memory_value),
            None => match self.disk_cache.get(key)? {
                Some(value) => {
                    let deserialized: V = postcard::from_bytes(&value)?;

                    self.memory_cache.insert(*key, deserialized.clone()).await;
                    Some(deserialized)
                }
                None => match self.pg.get(key).await? {
                    Some(value) => {
                        let deserialized: V = postcard::from_bytes(&value)?;

                        self.memory_cache.insert(*key, deserialized.clone()).await;
                        self.spawn_disk_cache_write_vec(*key, value).await?;

                        Some(deserialized)
                    }
                    None => None,
                },
            },
        })
    }

    pub async fn get_bulk(&self, keys: &[K]) -> LayerDbResult<HashMap<K, V>> {
        let mut found_keys = HashMap::new();
        let mut not_found: Vec<K> = vec![];

        for key in keys {
            if let Some(found) = match self.memory_cache.get(key).await {
                Some(memory_value) => Some(memory_value),
                None => match self.disk_cache.get(key)? {
                    Some(value) => {
                        let deserialized: V = postcard::from_bytes(&value)?;

                        self.memory_cache.insert(*key, deserialized.clone()).await;
                        Some(deserialized)
                    }
                    None => {
                        not_found.push(*key);
                        None
                    }
                },
            } {
                found_keys.insert(*key, found);
            }
        }

        if !not_found.is_empty() {
            if let Some(pg_found) = self.pg.get_many(&not_found).await? {
                for (k, v) in pg_found {
                    let deserialized: V = postcard::from_bytes(&v)?;
                    self.memory_cache.insert(k, deserialized.clone()).await;
                    found_keys.insert(k, deserialized);
                }
            }
        }

        Ok(found_keys)
    }

    pub fn memory_cache(&self) -> MemoryCache<K, V> {
        self.memory_cache.clone()
    }

    pub fn disk_cache(&self) -> Arc<DiskCache<K>> {
        self.disk_cache.clone()
    }

    pub fn pg(&self) -> PgLayer<K> {
        self.pg.clone()
    }

    pub async fn remove_from_memory(&self, key: K) {
        self.memory_cache.remove(&key).await;
    }

    pub async fn insert(&self, key: K, value: V) {
        if !self.memory_cache.contains(&key) {
            self.memory_cache.insert(key, value).await;
        }
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
