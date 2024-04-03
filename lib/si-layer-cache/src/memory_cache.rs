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
        Self {
            cache: Cache::new(u64::MAX),
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
