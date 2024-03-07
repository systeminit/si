//! A fast in-memory, network aware, layered write-through cache for System Initiative.
//!
//! It should have 3 layers of caching:
//!
//! * Moka, an in-memory LRU style cache.
//! * Sled, an on-disk memory-mapped cache, to keep more data locally than can be held in memory
//! * Postgres, our final persistant storage layer.
//!
//! When a write is requested, the following happens:
//!
//! * The data is written first to a Moka cache
//! * Then written to Sled for persistent storage
//! * The data is then published to a nats topic layer-cache.workspaceId
//! * Any remote si-layer-cache instances listen to this topic, and populate their local caches
//! * Postgres gets written to eventually by a 'persister' process that writes to PG from the write
//! stream
//!
//! When a read is requested, the following happen:
//!
//! * The data is read from the moka cache
//! * On a miss, the data is read from sled, inserted into Moka, and returned to the user
//! * On a miss, the data is read from Postgres, inserted into sled, inserted into moka, and
//! returned to the user
//!
//! The postgres bits remain unimplemented! :)

pub mod disk_cache;
pub mod error;
pub mod pg;

use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};
use si_data_pg::PgPool;
use std::{hash::Hash, sync::Arc};
use tokio::{sync::Mutex, task::JoinSet};

use disk_cache::DiskCache;
use error::LayerCacheResult;
use pg::PgLayer;

#[derive(Clone)]
pub struct LayerCache<K, V>
where
    K: AsRef<[u8]> + Copy + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    memory_cache: Arc<Box<dyn MemoryCacher<Key = K, Value = V>>>,
    disk_cache: Arc<DiskCache<K>>,
    pg: PgLayer<K>,
    join_set: Arc<Mutex<JoinSet<()>>>,
}

impl<K, V> LayerCache<K, V>
where
    K: AsRef<[u8]> + Copy + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn new(
        name: &str,
        memory_cache: Box<dyn MemoryCacher<Key = K, Value = V>>,
        fast_disk: sled::Db,
        pg_pool: PgPool,
    ) -> LayerCacheResult<Self> {
        let disk_cache = Arc::new(DiskCache::new(fast_disk, name.as_bytes())?);

        let pg = PgLayer::new(pg_pool);
        pg.migrate().await?;

        Ok(LayerCache {
            memory_cache: Arc::new(memory_cache),
            disk_cache,
            join_set: Arc::new(Mutex::new(JoinSet::new())),
            pg,
        })
    }

    pub async fn get(&self, key: &K) -> LayerCacheResult<Option<V>> {
        Ok(match self.memory_cache.get_value(key).await {
            Some(memory_value) => Some(memory_value),
            None => match self.disk_cache.get(key)? {
                Some(value) => {
                    let deserialized: V = postcard::from_bytes(&value)?;

                    self.memory_cache
                        .insert_value(*key, deserialized.clone())
                        .await;
                    Some(deserialized)
                }
                None => match self.pg.get(key).await? {
                    Some(value) => {
                        let deserialized: V = postcard::from_bytes(&value)?;
                        self.memory_cache
                            .insert_value(*key, deserialized.clone())
                            .await;

                        let self_clone = self.clone();
                        let key_clone = *key;
                        tokio::task::spawn(async move {
                            let _ = self_clone.disk_cache.insert(key_clone, &value);
                        });

                        Some(deserialized)
                    }
                    None => None,
                },
            },
        })
    }

    pub fn memory_cache(&self) -> Arc<Box<dyn MemoryCacher<Key = K, Value = V>>> {
        self.memory_cache.clone()
    }

    pub fn disk_cache(&self) -> Arc<DiskCache<K>> {
        self.disk_cache.clone()
    }

    pub async fn remove_from_memory(&self, key: K) {
        self.memory_cache.remove_value(&key).await;
    }

    /// The disk and database writers will spawn a thread to perform the write,
    /// and that thread must be joined on if all the writes are to succeed (the
    /// caller may terminate before the write threads). This method will block
    /// until all writes have succeeded.
    pub async fn join_all_write_tasks(&self) {
        while let Some(_) = self.join_set.lock().await.join_next().await {}
    }

    pub async fn insert(&self, key: K, value: V) -> LayerCacheResult<()> {
        let in_memory = self.memory_cache.has_key(&key);
        let on_disk = self.disk_cache.contains_key(&key)?;

        match (in_memory, on_disk) {
            // In memory and on disk, do nothing
            (true, true) => (),
            // Neither on memory or on disk
            (false, false) => {
                self.memory_cache.insert_value(key, value.clone()).await;
                let self_clone = self.clone();
                let value = value.clone();
                self.join_set.lock().await.spawn(async move {
                    // TODO: we are ignoring write failures to disk, but we
                    // should probably do something about them?
                    if let Ok(serialized) = postcard::to_stdvec(&value) {
                        let _ = self_clone.disk_cache.insert(key, &serialized);
                    }
                });
            }
            // Not in memory, but on disk - we can write, because objects are immutable
            (false, true) => {
                self.memory_cache.insert_value(key, value.clone()).await;
            }
            // In memory, but not on disk
            (true, false) => {
                let self_clone = self.clone();
                let value = value.clone();
                self.join_set.lock().await.spawn(async move {
                    if let Ok(serialized) = postcard::to_stdvec(&value) {
                        let _ = self_clone.disk_cache.insert(key, &serialized);
                    }
                });
            }
        }

        let self_clone = self.clone();
        self.join_set.lock().await.spawn(async move {
            if let Ok(serialized) = postcard::to_stdvec(&value) {
                let _ = self_clone.pg.insert(key, &serialized).await;
            }
        });

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait MemoryCacher: Send + Sync {
    type Key;
    type Value;

    async fn get_value(&self, key: &Self::Key) -> Option<Self::Value>;

    async fn insert_value(&self, key: Self::Key, value: Self::Value);

    async fn remove_value(&self, key: &Self::Key);

    fn has_key(&self, key: &Self::Key) -> bool;
}

#[async_trait::async_trait]
impl<K, V> MemoryCacher for Cache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    type Key = K;
    type Value = V;

    async fn get_value(&self, key: &Self::Key) -> Option<Self::Value> {
        self.get(key).await
    }

    async fn insert_value(&self, key: Self::Key, value: Self::Value) {
        self.insert(key, value).await;
    }

    async fn remove_value(&self, key: &Self::Key) {
        self.remove(key).await;
    }

    fn has_key(&self, key: &Self::Key) -> bool {
        self.contains_key(key)
    }
}
