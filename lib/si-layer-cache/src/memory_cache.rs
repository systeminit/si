use serde::Deserialize;
use std::{sync::Arc, time::Duration};

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
        Self::new(MemoryCacheConfig::default())
    }
}

impl<V> MemoryCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + Clone + 'static,
{
    pub fn new(config: MemoryCacheConfig) -> Self {
        Self {
            cache: Cache::builder()
                .time_to_idle(Duration::from_secs(config.seconds_to_idle))
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryCacheConfig {
    seconds_to_idle: u64,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            seconds_to_idle: 600,
        }
    }
}
