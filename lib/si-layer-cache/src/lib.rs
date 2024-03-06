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

use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;

use disk_cache::DiskCache;
use error::LayerCacheResult;

pub struct LayerCache<K, V>
where
    K: AsRef<[u8]> + Copy,
    V: Serialize + DeserializeOwned + Clone,
{
    pub memory_cache: Box<dyn MemoryCacher<Key = K, Value = V>>,
    pub disk_cache: DiskCache<K>,
}

impl<K, V> LayerCache<K, V>
where
    K: AsRef<[u8]> + Copy,
    V: Serialize + DeserializeOwned + Clone,
{
    pub fn new(
        db: sled::Db,
        name: &str,
        memory_cache: Box<dyn MemoryCacher<Key = K, Value = V>>,
    ) -> LayerCacheResult<Self> {
        let disk_cache = DiskCache::new(db, name.as_bytes())?;
        Ok(LayerCache {
            memory_cache,
            disk_cache,
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
                None => None,
            },
        })
    }

    pub fn memory_cache(&self) -> &dyn MemoryCacher<Key = K, Value = V> {
        self.memory_cache.as_ref()
    }

    pub fn disk_cache(&self) -> &DiskCache<K> {
        &self.disk_cache
    }

    pub async fn remove_from_memory(&self, key: K) {
        self.memory_cache.remove_value(&key).await;
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
                let serialized = postcard::to_stdvec(&value)?;
                self.disk_cache.insert(key, &serialized)?;
            }
            // Not in memory, but on disk - we can write, because objects are immutable
            (false, true) => {
                self.memory_cache.insert_value(key, value).await;
            }
            // In memory, but not on disk
            (true, false) => {
                let serialized = postcard::to_stdvec(&value)?;
                self.disk_cache.insert(key, &serialized)?;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait MemoryCacher {
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
