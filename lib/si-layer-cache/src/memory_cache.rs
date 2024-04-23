use std::sync::Arc;

use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone, Debug)]
pub struct MemoryCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + Clone + 'static,
{
    cache: Cache<Arc<str>, V>,
}

impl<V> Default for MemoryCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> MemoryCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + Clone + 'static,
{
    pub fn new() -> Self {
        // hardcoding max cache size to 12gb as a hammer to ensure we don't starve the OS
        // TODO(scott): make this dynamic based on the the total memory set
        Self {
            cache: Cache::new(12 * 1024 * 1024 * 1024),
        }
    }

    pub async fn get(&self, key: &str) -> Option<V> {
        self.cache.get(key).await
    }

    pub async fn insert(&self, key: Arc<str>, value: V) {
        self.cache.insert(key, value).await;
    }

    pub async fn remove(&self, key: &str) {
        self.cache.remove(key).await;
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }
}
