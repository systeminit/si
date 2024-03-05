use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::CacheType;

pub type CacheKeyRef = [u8];
pub type CacheKey = Vec<u8>;
pub type CacheValueRaw = Vec<u8>;
pub type CacheValue = Arc<Vec<u8>>;

pub struct MemoryCache {
    pub object_cache: Cache<CacheKey, CacheValue>,
    pub graph_cache: Cache<CacheKey, CacheValue>,
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryCache {
    pub fn new() -> MemoryCache {
        MemoryCache {
            object_cache: Cache::builder()
                .max_capacity(10000)
                .time_to_idle(Duration::from_secs(36000))
                .build(),
            graph_cache: Cache::builder()
                .max_capacity(10000)
                .time_to_idle(Duration::from_secs(36000))
                .build(),
        }
    }

    fn get_cache(&self, cache_type: &CacheType) -> &Cache<CacheKey, CacheValue> {
        match cache_type {
            CacheType::Object => &self.object_cache,
            CacheType::Graph => &self.graph_cache,
        }
    }

    pub async fn insert(
        &self,
        cache_type: &CacheType,
        key: impl Into<CacheKey>,
        value: impl Into<Vec<u8>>,
    ) {
        let cache = self.get_cache(cache_type);
        cache.insert(key.into(), Arc::new(value.into())).await;
    }

    pub async fn get(
        &self,
        cache_type: &CacheType,
        key: impl AsRef<CacheKeyRef>,
    ) -> Option<CacheValue> {
        let cache = self.get_cache(cache_type);
        let key = key.as_ref();
        cache.get(key).await
    }

    pub fn contains_key(&self, cache_type: &CacheType, key: impl AsRef<CacheKeyRef>) -> bool {
        let cache = self.get_cache(cache_type);
        let key = key.as_ref();
        cache.contains_key(key)
    }
}
