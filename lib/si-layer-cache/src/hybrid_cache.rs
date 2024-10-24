use foyer::{DirectFsDeviceOptions, Engine, EvictionConfig, HybridCache, HybridCacheBuilder};
use std::sync::Arc;
use std::{hash::Hash, path::PathBuf};
use telemetry::tracing::error;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::db::serialize;
use crate::error::LayerDbResult;

#[derive(Clone, Debug, Deserialize, Serialize)]
enum MaybeDeserialized<V>
where
    V: Serialize + Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
{
    RawBytes(Vec<u8>),
    DeserializedValue(V),
}

#[derive(Clone, Debug)]
pub struct Cache<V>
where
    V: Serialize + DeserializeOwned + Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
{
    cache: HybridCache<Arc<str>, MaybeDeserialized<V>>,
}

// impl<V> Default for HybridCache<V>
// where
//     V: Serialize + DeserializeOwned + Clone + Eq + Send + Sync + 'static + std::hash::Hash,
// {
//     fn default() -> Self {
//         Self::new(HybridCacheConfig::default())
//     }
// }
//
impl<V> Cache<V>
where
    V: Serialize + DeserializeOwned + Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
{
    pub async fn new(config: HybridCacheConfig) -> LayerDbResult<Self> {
        // todo: unwrapping
        Ok(Self {
            cache: HybridCacheBuilder::new()
                .with_name(&config.name)
                .memory(config.memory as usize)
                .storage(Engine::Large)
                .with_device_options(DirectFsDeviceOptions::new(config.disk))
                .build()
                .await
                .unwrap(),
        })
    }

    pub async fn get(&self, key: &str) -> Option<V> {
        match self.cache.obtain(key.into()).await {
            Ok(Some(entry)) => match entry.value() {
                // todo: bad clone here
                MaybeDeserialized::DeserializedValue(v) => Some(v.clone()),
                MaybeDeserialized::RawBytes(bytes) => {
                    // If we fail to deserialize the raw bytes for some reason, pretend that we never
                    // had the key in the first place, and also remove it from the cache.
                    match serialize::from_bytes_async::<V>(bytes).await {
                        Ok(deserialized) => {
                            self.insert(key.into(), deserialized.clone());
                            Some(deserialized)
                        }
                        Err(e) => {
                            error!(
                        "Failed to deserialize stored bytes from memory cache for key ({:?}): {}",
                        key,
                        e
                    );
                            self.remove(key);
                            None
                        }
                    }
                }
            },

            _ => None,
        }
    }

    pub fn insert(&self, key: Arc<str>, value: V) {
        self.cache
            .insert(key, MaybeDeserialized::DeserializedValue(value));
    }
    //
    //     pub async fn insert_raw_bytes(&self, key: Arc<str>, raw_bytes: Vec<u8>) {
    //         self.cache
    //             .insert(key, MaybeDeserialized::RawBytes(raw_bytes))
    //             .await;
    //     }

    pub fn remove(&self, key: &str) {
        self.cache.remove(key);
    }
}

//     pub fn contains(&self, key: &str) -> bool {
//         self.cache.contains_key(key)
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HybridCacheConfig {
    disk: PathBuf,
    memory: u64,
    name: String,
}

impl Default for HybridCacheConfig {
    fn default() -> Self {
        let sys = sysinfo::System::new();
        let path = tempfile::TempDir::with_prefix_in("default-cache-", "/tmp")
            .expect("unable to create tmp dir for layerdb")
            .into_path();
        Self {
            disk: path,
            memory: sys.total_memory() - 536870912, //reserve 512mb for OS
            name: "default".to_string(),
        }
    }
}
