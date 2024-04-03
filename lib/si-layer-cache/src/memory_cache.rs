use std::{sync::Arc, time::Duration};

use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};

const DEFAULT_SIZE: u64 = 65_536 * 8;
const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 2);
const DEFAULT_TTI: Duration = Duration::from_secs(60 * 60 * 24);

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
            cache: Cache::builder()
                .max_capacity(DEFAULT_SIZE)
                .time_to_idle(DEFAULT_TTI)
                .time_to_live(DEFAULT_TTL)
                .build(),
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
