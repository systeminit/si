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
pub mod memory_cache;

use std::fmt;
use std::path::Path;

use memory_cache::{CacheKey, CacheValueRaw};

use crate::disk_cache::DiskCache;
use crate::error::LayerCacheResult;
use crate::memory_cache::{CacheKeyRef, CacheValue, MemoryCache};

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub enum CacheType {
    Object = 1,
    Graph,
}

impl fmt::Display for CacheType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CacheType::Object => write!(f, "object"),
            CacheType::Graph => write!(f, "graph"),
        }
    }
}

pub struct LayerCache {
    pub memory_cache: MemoryCache,
    pub disk_cache: DiskCache,
}

impl LayerCache {
    pub fn new(path: impl AsRef<Path>) -> LayerCacheResult<LayerCache> {
        let memory_cache = MemoryCache::new();
        let disk_cache = DiskCache::new(path)?;
        Ok(LayerCache {
            memory_cache,
            disk_cache,
        })
    }

    #[inline]
    pub async fn get(
        &self,
        cache_type: &CacheType,
        key: impl AsRef<CacheKeyRef>,
    ) -> LayerCacheResult<Option<CacheValue>> {
        let key = key.as_ref();
        let memory_value = self.memory_cache.get(cache_type, key).await;
        if memory_value.is_some() {
            Ok(memory_value)
        } else {
            let maybe_value = self.disk_cache.get(cache_type, key)?;
            match maybe_value {
                Some(value) => {
                    let d: Vec<u8> = value.as_ref().into();
                    self.memory_cache
                        .insert(cache_type, Vec::from(key), d)
                        .await;
                    Ok(self.memory_cache.get(cache_type, key).await)
                }
                None => Ok(None),
            }
        }
    }

    #[inline]
    pub async fn insert(
        &self,
        cache_type: &CacheType,
        key: impl Into<CacheKey>,
        value: impl Into<CacheValueRaw>,
    ) -> LayerCacheResult<()> {
        let key = key.into();
        let in_memory = self.memory_cache.contains_key(cache_type, &key);
        let on_disk = self.disk_cache.contains_key(cache_type, &key)?;

        match (in_memory, on_disk) {
            // In memory and on disk
            (true, true) => Ok(()),
            // Neither on memory or on disk
            (false, false) => {
                let value = value.into();
                self.memory_cache
                    .insert(cache_type, key.clone(), value.clone())
                    .await;
                self.disk_cache.insert(cache_type, key, value)?;
                Ok(())
            }
            // Not in memory, but on disk - we can write, becasue objects are immutable
            (false, true) => {
                let value = value.into();
                self.memory_cache
                    .insert(cache_type, key.clone(), value)
                    .await;
                Ok(())
            }
            // In memory, but not on disk
            (true, false) => {
                let value = value.into();
                self.disk_cache.insert(cache_type, key, value)?;
                Ok(())
            }
        }
    }
}
