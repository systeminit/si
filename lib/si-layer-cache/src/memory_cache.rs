use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use telemetry::prelude::*;

use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};

use crate::db::serialize;

#[derive(Clone, Debug)]
enum MaybeDeserialized<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    RawBytes(Vec<u8>),
    DeserializedValue(V),
}

#[derive(Clone, Debug)]
pub struct MemoryCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cache: Cache<Arc<str>, MaybeDeserialized<V>>,
}

impl<V> Default for MemoryCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new(MemoryCacheConfig::default())
    }
}

impl<V> MemoryCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(config: MemoryCacheConfig) -> Self {
        Self {
            cache: Cache::builder()
                .time_to_idle(Duration::from_secs(config.seconds_to_idle))
                .build(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<V> {
        match self.cache.get(key).await {
            Some(MaybeDeserialized::DeserializedValue(value)) => Some(value),
            Some(MaybeDeserialized::RawBytes(bytes)) => {
                // If we fail to deserialize the raw bytes for some reason, pretend that we never
                // had the key in the first place, and also remove it from the cache.
                match serialize::from_bytes_async::<V>(&bytes).await {
                    Ok(deserialized) => {
                        self.insert(key.into(), deserialized.clone()).await;
                        Some(deserialized)
                    }
                    Err(e) => {
                        error!(
                            "Failed to deserialize stored bytes from memory cache for key ({:?}): {}",
                            key,
                            e
                        );
                        self.remove(key).await;
                        None
                    }
                }
            }
            None => None,
        }
    }

    pub async fn insert(&self, key: Arc<str>, value: V) {
        self.cache
            .insert(key, MaybeDeserialized::DeserializedValue(value))
            .await;
    }

    pub async fn insert_raw_bytes(&self, key: Arc<str>, raw_bytes: Vec<u8>) {
        self.cache
            .insert(key, MaybeDeserialized::RawBytes(raw_bytes))
            .await;
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
