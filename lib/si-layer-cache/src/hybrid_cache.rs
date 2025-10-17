use std::{
    cmp::max,
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Arc,
        LazyLock,
    },
};

use foyer::{
    DirectFsDeviceOptions,
    Engine,
    FifoPicker,
    HybridCache,
    HybridCacheBuilder,
    LargeEngineOptions,
    RateLimitPicker,
    RecoverMode,
    TokioRuntimeOptions,
};
use mixtrics::registry::opentelemetry_0_26::OpenTelemetryMetricsRegistry;
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use telemetry::{
    opentelemetry::global,
    tracing::{
        error,
        info,
    },
};
use tokio::fs;

use crate::{
    LayerDbError,
    db::serialize,
    error::LayerDbResult,
};

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
const DEFAULT_DISK_RECOVER_CONCURRENCY: usize = 8;

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
    DeserializedValue { value: V, size_hint: usize },
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

        let cache_meter_name: &'static str = config.name.clone().leak();

        let builder = HybridCacheBuilder::new()
            .with_name(config.name.clone())
            .with_metrics_registry(Box::new(OpenTelemetryMetricsRegistry::new(global::meter(
                cache_meter_name,
            ))))
            .memory(memory_cache_capacity_bytes)
            .with_weighter(
                |_key: &Arc<str>, value: &MaybeDeserialized<V>| match value {
                    MaybeDeserialized::RawBytes(bytes) => bytes.len(),
                    MaybeDeserialized::DeserializedValue { size_hint, .. } => *size_hint,
                },
            )
            .storage(Engine::Large)
            .with_runtime_options(foyer::RuntimeOptions::Unified(TokioRuntimeOptions {
                max_blocking_threads: 0,
                worker_threads: 0,
            }))
            .with_admission_picker(Arc::new(RateLimitPicker::new(
                config.disk_admission_rate_limit,
            )))
            .with_large_object_disk_cache_options(
                LargeEngineOptions::new()
                    .with_buffer_pool_size(config.disk_buffer_size)
                    .with_eviction_pickers(vec![Box::<FifoPicker>::default()])
                    .with_flushers(config.disk_buffer_flushers)
                    .with_recover_concurrency(config.disk_recover_concurrency)
                    .with_indexer_shards(config.disk_indexer_shards)
                    .with_reclaimers(config.disk_reclaimers),
            )
            .with_recover_mode(RecoverMode::Quiet);

        let builder = if config.disk_layer {
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
                cache.disk.layer = config.disk_layer,
                cache.disk.total_bytes = total_disk_bytes,
                cache.disk.size_bytes = disk_cache_capacity_bytes,
                cache.disk.reserved_percent = config.disk_reserved_percent,
                cache.disk.usable_max_percent = config.disk_usable_max_percent,
                cache.disk.rate_limit = config.disk_admission_rate_limit,
                cache.memory.total_bytes = total_memory_bytes,
                cache.memory.size_bytes = memory_cache_capacity_bytes,
                cache.memory.reserved_percent = config.memory_reserved_percent,
                cache.memory.usable_max_percent = config.memory_usable_max_percent,
                "creating cache",
            );

            builder.with_device_options(
                DirectFsDeviceOptions::new(config.disk_path)
                    .with_capacity(disk_cache_capacity_bytes),
            )
        } else {
            info!(
                cache.name = &config.name,
                cache.disk.layer = config.disk_layer,
                cache.disk.reserved_percent = config.disk_reserved_percent,
                cache.disk.usable_max_percent = config.disk_usable_max_percent,
                cache.disk.rate_limit = config.disk_admission_rate_limit,
                cache.memory.total_bytes = total_memory_bytes,
                cache.memory.size_bytes = memory_cache_capacity_bytes,
                cache.memory.reserved_percent = config.memory_reserved_percent,
                cache.memory.usable_max_percent = config.memory_usable_max_percent,
                "creating cache",
            );

            builder
        };

        let cache: HybridCache<Arc<str>, MaybeDeserialized<V>> = builder
            .build()
            .await
            .map_err(|e| LayerDbError::Foyer(e.into()))?;

        Ok(Self { cache })
    }

    pub async fn get(&self, key: Arc<str>) -> Option<V> {
        if let Ok(Some(entry)) = self.cache.obtain(key.clone()).await {
            return self.maybe_deserialize(key, entry.value().clone()).await;
        }
        None
    }

    pub async fn get_from_memory(&self, key: Arc<str>) -> Option<V> {
        if let Some(entry) = self.cache.memory().get(&key) {
            return self.maybe_deserialize(key, entry.value().clone()).await;
        }
        None
    }

    async fn maybe_deserialize(&self, key: Arc<str>, entry: MaybeDeserialized<V>) -> Option<V> {
        match entry {
            MaybeDeserialized::DeserializedValue { value, .. } => Some(value.clone()),
            MaybeDeserialized::RawBytes(bytes) => {
                // If we fail to deserialize the raw bytes for some reason, pretend that we never
                // had the key in the first place, and also remove it from the cache.
                match serialize::from_bytes_async::<V>(&bytes).await {
                    Ok(deserialized) => {
                        self.insert(key, deserialized.clone(), bytes.len());
                        Some(deserialized)
                    }
                    Err(e) => {
                        error!(
                            "Failed to deserialize stored bytes from memory cache for key ({:?}): {}",
                            key, e
                        );
                        self.remove(&key);
                        None
                    }
                }
            }
        }
    }

    pub fn insert(&self, key: Arc<str>, value: V, size_hint: usize) {
        self.cache.insert(
            key,
            MaybeDeserialized::DeserializedValue { value, size_hint },
        );
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
    disk_layer: bool,
    disk_reserved_percent: u8,
    disk_usable_max_percent: u8,
    disk_admission_rate_limit: usize,
    disk_buffer_size: usize,
    disk_buffer_flushers: usize,
    disk_indexer_shards: usize,
    disk_path: PathBuf,
    disk_reclaimers: usize,
    disk_recover_concurrency: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let disk_path = tempfile::TempDir::with_prefix("default-cache-")
            .expect("unable to create tmp dir for layerdb")
            .path()
            .to_path_buf();

        Self {
            name: "default".to_string(),
            memory_reserved_percent: DEFAULT_MEMORY_RESERVED_PERCENT,
            memory_usable_max_percent: DEFAULT_MEMORY_USABLE_MAX_PERCENT,
            disk_layer: true,
            disk_reserved_percent: DEFAULT_DISK_RESERVED_PERCENT,
            disk_usable_max_percent: DEFAULT_DISK_USAGE_MAX_PERCENT,
            disk_admission_rate_limit: DEFAULT_DISK_CACHE_RATE_LIMIT,
            disk_buffer_size: DEFAULT_DISK_BUFFER_SIZE,
            disk_buffer_flushers: DEFAULT_DISK_BUFFER_FLUSHERS,
            disk_indexer_shards: DEFAULT_DISK_INDEXER_SHARDS,
            disk_path,
            disk_reclaimers: DEFAULT_DISK_RECLAIMERS,
            disk_recover_concurrency: DEFAULT_DISK_RECOVER_CONCURRENCY,
        }
    }
}

impl CacheConfig {
    /// Returns the size of system memory, in bytes.
    #[inline]
    pub fn total_system_memory_bytes() -> u64 {
        *TOTAL_SYSTEM_MEMORY_BYTES
    }

    /// Updates the disk layer strategy (i.e. whether to use a disk cache layer or not).
    pub fn disk_layer(mut self, value: bool) -> Self {
        self.disk_layer = value;
        self
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

    /// Returns the disk path for the cache
    pub fn disk_path(&self) -> &Path {
        &self.disk_path
    }
}
