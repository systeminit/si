use std::hash::Hash;

use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone, Debug)]
pub struct MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + AsRef<[u8]> + Copy + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + Clone + 'static,
{
    cache: Cache<K, V>,
}

impl<K, V> Default for MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + AsRef<[u8]> + Copy + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + AsRef<[u8]> + Copy + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + Clone + 'static,
{
    pub fn new() -> Self {
        Self {
            cache: Cache::new(u64::MAX),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).await
    }

    pub async fn insert(&self, key: K, value: V) {
        self.cache.insert(key, value).await;
    }

    pub async fn remove(&self, key: &K) {
        self.cache.remove(key).await;
    }

    pub fn contains(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }
}
