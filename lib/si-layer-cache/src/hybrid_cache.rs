use foyer::{
    DirectFsDeviceOptions, Engine, FifoPicker, HybridCache, HybridCacheBuilder, LargeEngineOptions,
    RateLimitPicker, RecoverMode,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use telemetry::tracing::error;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::db::serialize;
use crate::error::LayerDbResult;
use crate::LayerDbError;

const DEFAULT_DISK_CACHE_RATE_LIMIT: usize = 1024 * 1024 * 1024;
const DEFAULT_DISK_BUFFER_SIZE: usize = 1024 * 1024 * 128; // 128mb
const DEFAULT_DISK_BUFFER_FLUSHERS: usize = 2;
const DEFAULT_DISK_CAPACITY: usize = 1024 * 1024 * 1024 * 16; // 16gb

#[derive(Clone, Debug, Deserialize, Serialize)]
enum MaybeDeserialized<V>
where
    V: Serialize + Clone + Send + Sync + 'static,
{
    RawBytes(Vec<u8>),
    DeserializedValue(V),
}

#[derive(Clone, Debug)]
pub struct Cache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cache: HybridCache<Arc<str>, MaybeDeserialized<V>>,
}

impl<V> Cache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn new(config: CacheConfig) -> LayerDbResult<Self> {
        let cache: HybridCache<Arc<str>, MaybeDeserialized<V>> = HybridCacheBuilder::new()
            .memory((config.memory as f32 * config.memory_percentage) as usize)
            .with_weighter(|_key: &Arc<str>, value: &MaybeDeserialized<V>| size_of_val(value))
            .storage(Engine::Large)
            .with_admission_picker(Arc::new(RateLimitPicker::new(
                config.disk_admission_rate_limit,
            )))
            .with_device_options(
                DirectFsDeviceOptions::new(config.disk_path).with_capacity(config.disk_capacity),
            )
            .with_large_object_disk_cache_options(
                LargeEngineOptions::new()
                    .with_flushers(config.disk_buffer_flushers)
                    .with_buffer_pool_size(config.disk_buffer_size)
                    .with_eviction_pickers(vec![Box::<FifoPicker>::default()]),
            )
            .with_recover_mode(RecoverMode::Quiet)
            .build()
            .await
            .map_err(|e| LayerDbError::Foyer(e.into()))?;

        Ok(Self { cache })
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

    pub fn insert_raw_bytes(&self, key: Arc<str>, raw_bytes: Vec<u8>) {
        self.cache
            .insert(key, MaybeDeserialized::RawBytes(raw_bytes));
    }

    pub fn remove(&self, key: &str) {
        self.cache.remove(key);
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains(key)
    }

    pub async fn close(&self) -> LayerDbResult<()> {
        self.cache
            .close()
            .await
            .map_err(|e| LayerDbError::Foyer(e.into()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheConfig {
    disk_capacity: usize,
    disk_path: PathBuf,
    disk_admission_rate_limit: usize,
    disk_buffer_size: usize,
    disk_buffer_flushers: usize,
    memory: u64,
    memory_percentage: f32,
    name: String,
}

impl CacheConfig {
    // set the total size of the disk cache for this instance of the cache
    pub fn with_disk_capacity(mut self, capacity: usize) -> Self {
        self.disk_capacity = capacity;
        self
    }

    // set the percentage of total memory
    pub fn with_memory_percentage(mut self, memory_percentage: f32) -> Self {
        self.memory_percentage = memory_percentage;
        self
    }

    // append an additional path to the existing disk path
    pub fn with_path_join(mut self, path: impl AsRef<Path>) -> Self {
        self.disk_path = self.disk_path.join(path);
        self
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        let sys = sysinfo::System::new_all();
        let path = tempfile::TempDir::with_prefix_in("default-cache-", "/tmp")
            .expect("unable to create tmp dir for layerdb")
            .path()
            .to_path_buf();

        Self {
            disk_capacity: DEFAULT_DISK_CAPACITY,
            disk_path: path,
            disk_admission_rate_limit: DEFAULT_DISK_CACHE_RATE_LIMIT,
            disk_buffer_size: DEFAULT_DISK_BUFFER_SIZE,
            disk_buffer_flushers: DEFAULT_DISK_BUFFER_FLUSHERS,
            memory: sys.total_memory(),
            memory_percentage: 1.0,
            name: "default".to_string(),
        }
    }
}
