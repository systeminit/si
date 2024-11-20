use foyer::{
    DirectFsDeviceOptions, Engine, FifoPicker, HybridCache, HybridCacheBuilder, LargeEngineOptions,
    RateLimitPicker, RecoverMode,
};
use std::cmp::max;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use telemetry::tracing::{error, info};
use tokio::fs;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::db::serialize;
use crate::error::LayerDbResult;
use crate::LayerDbError;

const FOYER_DISK_CACHE_MINUMUM: u64 = 1024 * 1024 * 1024; // 1gb
const DEFAULT_MEMORY_RESERVED_PERCENT: u8 = 40;
const DEFAULT_MEMORY_USABLE_MAX_PERCENT: u8 = 100;
const DEFAULT_DISK_RESERVED_PERCENT: u8 = 5;
const DEFAULT_DISK_USAGE_MAX_PERCENT: u8 = 100;
const DEFAULT_DISK_CACHE_RATE_LIMIT: usize = 1024 * 1024 * 1024;
const DEFAULT_DISK_BUFFER_SIZE: usize = 1024 * 1024 * 128; // 128mb
const DEFAULT_DISK_BUFFER_FLUSHERS: usize = 2;
const DEFAULT_DISK_INDEXER_SHARDS: usize = 64;
const DEFAULT_DISK_RECLAIMERS: usize = 2;

static TOTAL_SYSTEM_MEMORY_BYTES: LazyLock<u64> = LazyLock::new(|| {
    let sys = sysinfo::System::new_all();
    sys.total_memory()
});

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
        let total_memory_bytes = *TOTAL_SYSTEM_MEMORY_BYTES;

        let memory_cache_capacity_bytes = {
            // Subtract reserved memory percentage to determine total usable cache memory
            let total_usable_memory_bytes = (total_memory_bytes as f64
                * (1.0 - (config.memory_reserved_percent as f64 / 100.0)))
                .floor() as u64;
            // Compute final usable memory as a percentage of the maximum usable memory
            let computed_memory_cache_capacity_bytes = (total_usable_memory_bytes as f64
                * (config.memory_usable_max_percent as f64 / 100.0))
                .floor() as u64;

            computed_memory_cache_capacity_bytes.try_into()?
        };

        fs::create_dir_all(config.disk_path.as_path()).await?;
        // Compute total disk which is in use for `disk_path`
        let total_disk_bytes = fs4::total_space(config.disk_path.as_path())?;

        let disk_cache_capacity_bytes = {
            // Subtract reserved disk percentage to determine total usable cache disk
            let total_usable_disk_bytes = (total_disk_bytes as f64
                * (1.0 - (config.disk_reserved_percent as f64 / 100.0)))
                .floor() as u64;
            // Compute final usable disk as a percentage of the maximum usable disk
            let computed_disk_cache_capacity_bytes = (total_usable_disk_bytes as f64
                * (config.disk_usable_max_percent as f64 / 100.0))
                .floor() as u64;

            // Ensure that the computed value is at least as big as the Foyer minimum
            max(computed_disk_cache_capacity_bytes, FOYER_DISK_CACHE_MINUMUM).try_into()?
        };

        info!(
            cache.name = &config.name,
            cache.disk.total_bytes = total_disk_bytes,
            cache.disk.size_bytes = disk_cache_capacity_bytes,
            cache.disk.reserved_percent = config.disk_reserved_percent,
            cache.disk.usable_max_percent = config.disk_usable_max_percent,
            cache.memory.total_bytes = total_memory_bytes,
            cache.memory.size_bytes = memory_cache_capacity_bytes,
            cache.memory.reserved_percent = config.memory_reserved_percent,
            cache.memory.usable_max_percent = config.memory_usable_max_percent,
            "creating cache",
        );

        let cache: HybridCache<Arc<str>, MaybeDeserialized<V>> = HybridCacheBuilder::new()
            .with_name(&config.name)
            .memory(memory_cache_capacity_bytes)
            .with_weighter(|_key: &Arc<str>, value: &MaybeDeserialized<V>| size_of_val(value))
            .storage(Engine::Large)
            .with_admission_picker(Arc::new(RateLimitPicker::new(
                config.disk_admission_rate_limit,
            )))
            .with_device_options(
                DirectFsDeviceOptions::new(config.disk_path)
                    .with_capacity(disk_cache_capacity_bytes),
            )
            .with_large_object_disk_cache_options(
                LargeEngineOptions::new()
                    .with_buffer_pool_size(config.disk_buffer_size)
                    .with_eviction_pickers(vec![Box::<FifoPicker>::default()])
                    .with_flushers(config.disk_buffer_flushers)
                    .with_indexer_shards(config.disk_indexer_shards)
                    .with_reclaimers(config.disk_reclaimers),
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
    name: String,
    memory_reserved_percent: u8,
    memory_usable_max_percent: u8,
    disk_reserved_percent: u8,
    disk_usable_max_percent: u8,
    disk_admission_rate_limit: usize,
    disk_buffer_size: usize,
    disk_buffer_flushers: usize,
    disk_indexer_shards: usize,
    disk_path: PathBuf,
    disk_reclaimers: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let disk_path = tempfile::TempDir::with_prefix_in("default-cache-", "/tmp")
            .expect("unable to create tmp dir for layerdb")
            .path()
            .to_path_buf();

        Self {
            name: "default".to_string(),
            memory_reserved_percent: DEFAULT_MEMORY_RESERVED_PERCENT,
            memory_usable_max_percent: DEFAULT_MEMORY_USABLE_MAX_PERCENT,
            disk_reserved_percent: DEFAULT_DISK_RESERVED_PERCENT,
            disk_usable_max_percent: DEFAULT_DISK_USAGE_MAX_PERCENT,
            disk_admission_rate_limit: DEFAULT_DISK_CACHE_RATE_LIMIT,
            disk_buffer_size: DEFAULT_DISK_BUFFER_SIZE,
            disk_buffer_flushers: DEFAULT_DISK_BUFFER_FLUSHERS,
            disk_indexer_shards: DEFAULT_DISK_INDEXER_SHARDS,
            disk_path,
            disk_reclaimers: DEFAULT_DISK_RECLAIMERS,
        }
    }
}

impl CacheConfig {
    /// Returns the size of system memory, in bytes.
    #[inline]
    pub fn total_system_memory_bytes() -> u64 {
        *TOTAL_SYSTEM_MEMORY_BYTES
    }

    // Updates the name for the cache (only used in logs for now).
    pub fn with_name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    /// Updates the reserve percentage of memory which will *never* be used for the cache.
    ///
    /// Default is `40`%.
    pub fn memory_reserved_percent(mut self, value: u8) -> Self {
        self.memory_reserved_percent = value;
        self
    }

    /// Updates the maximum percentage of usable memory to use for the cache.
    ///
    /// Default is `100`%.
    ///
    /// Note that this percentage does *not* include the reserved percentage.
    pub fn memory_usable_max_percent(mut self, value: u8) -> Self {
        self.memory_usable_max_percent = value;
        self
    }

    /// Updates the reserved percentage of the disk which will *never* be used for the cache.
    ///
    /// Default is `5`%.
    pub fn disk_reserved_percent(mut self, value: u8) -> Self {
        self.disk_reserved_percent = value;
        self
    }

    /// Updates the maximum percentage of the usable disk to use for the cache.
    ///
    /// Default is `100`%.
    ///
    /// Note that this percentage does *not* include the reserved percentage.
    pub fn disk_usable_max_percent(mut self, value: u8) -> Self {
        self.disk_usable_max_percent = value;
        self
    }

    /// Appends an additional path to the existing disk path
    pub fn with_path_join(mut self, path: impl AsRef<Path>) -> Self {
        self.disk_path = self.disk_path.join(path);
        self
    }
}
