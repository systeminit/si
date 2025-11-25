use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    str::FromStr,
    sync::Arc,
};

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use si_data_pg::PgPool;
use si_runtime::DedicatedExecutor;
use telemetry::prelude::*;
use telemetry_utils::monotonic;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use crate::{
    BackendType,
    LayerDbError,
    db::serialize,
    error::LayerDbResult,
    hybrid_cache::{
        Cache,
        CacheConfig,
    },
    persister::PersisterMode,
    pg::PgLayer,
    s3::S3Layer,
};

#[derive(Debug, Clone)]
pub struct LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    cache: Cache<V>,
    name: String,
    pg: PgLayer,
    #[allow(dead_code)]
    compute_executor: DedicatedExecutor,
    // NEW fields
    s3_layers: Option<Arc<HashMap<&'static str, S3Layer>>>,
    mode: PersisterMode,
}

impl<V> LayerCache<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        name: &str,
        pg_pool: PgPool,
        cache_config: CacheConfig,
        #[allow(dead_code)] compute_executor: DedicatedExecutor,
        tracker: TaskTracker,
        token: CancellationToken,
        s3_layers: Option<Arc<HashMap<&'static str, S3Layer>>>,
        mode: PersisterMode,
    ) -> LayerDbResult<Arc<Self>> {
        let cache = Cache::new(cache_config).await?;

        let pg = PgLayer::new(pg_pool.clone(), name);

        let lc: Arc<LayerCache<V>> = LayerCache {
            cache,
            name: name.to_string(),
            pg,
            compute_executor,
            s3_layers,
            mode,
        }
        .into();

        // Log cache initialization with persister mode and key transform strategy
        match lc.mode {
            PersisterMode::PostgresOnly => {
                info!(
                    cache.name = lc.name,
                    cache.persister_mode = ?lc.mode,
                    "layer cache initialized"
                );
            }
            PersisterMode::DualWrite | PersisterMode::S3Primary | PersisterMode::S3Only => {
                // For S3-enabled modes, also log the key transform strategy
                if let Some(s3_layers) = &lc.s3_layers {
                    if let Some(s3_layer) = s3_layers.get(lc.name.as_str()) {
                        info!(
                            cache.name = lc.name,
                            cache.persister_mode = ?lc.mode,
                            cache.key_transform_strategy = ?s3_layer.strategy(),
                            "layer cache initialized"
                        );
                    } else {
                        // S3 layer not found for this cache - this shouldn't happen in normal operation
                        // but log it without the strategy if it does
                        info!(
                            cache.name = lc.name,
                            cache.persister_mode = ?lc.mode,
                            "layer cache initialized (S3 layer not found)"
                        );
                    }
                } else {
                    // S3 not configured despite S3 mode - this shouldn't happen in normal operation
                    info!(
                        cache.name = lc.name,
                        cache.persister_mode = ?lc.mode,
                        "layer cache initialized (S3 not configured)"
                    );
                }
            }
        }

        tracker.spawn(lc.clone().shutdown_handler(token.clone()));
        Ok(lc)
    }

    async fn shutdown_handler(self: Arc<Self>, token: CancellationToken) -> LayerDbResult<()> {
        token.cancelled().await;
        debug!("shutting down layer cache {}", self.name);
        // foyer will wait on all outstanding flush and reclaim threads here
        self.cache().close().await?;
        Ok(())
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Option<V>> {
        use std::time::Instant;

        use telemetry_utils::histogram;

        let request_start = Instant::now();

        monotonic!(
            layer_cache_requests_total = 1,
            cache_name = self.name.as_str()
        );

        // Try memory/disk cache first
        let foyer_start = Instant::now();
        if let Some(value) = self.cache.get(key.clone()).await {
            histogram!(
                layer_cache.read_latency_ms = foyer_start.elapsed().as_millis() as f64,
                cache_name = self.name.as_str(),
                backend = "foyer",
                result = "hit"
            );

            monotonic!(
                layer_cache_backend_resolved = 1,
                cache_name = self.name.as_str(),
                backend = "foyer",
                result = "hit"
            );
            debug!(
                cache.name = self.name.as_str(),
                cache.key = key.as_ref(),
                "cache hit in memory/disk"
            );

            // Emit end-to-end metric for Foyer hit
            histogram!(
                layer_cache.request_latency_ms = request_start.elapsed().as_millis() as f64,
                cache_name = self.name.as_str(),
                result = "hit"
            );

            return Ok(Some(value));
        }

        debug!(
            cache.name = self.name.as_str(),
            cache.key = key.as_ref(),
            cache.mode = ?self.mode,
            "cache miss in memory/disk, fetching from backend"
        );

        // Emit Foyer miss metric
        histogram!(
            layer_cache.read_latency_ms = foyer_start.elapsed().as_millis() as f64,
            cache_name = self.name.as_str(),
            backend = "foyer",
            result = "miss"
        );

        // Cache miss - fetch from storage backend based on mode
        let bytes = match self.mode {
            PersisterMode::PostgresOnly | PersisterMode::DualWrite => {
                // Read from PG
                let result = self.pg.get(&key).await?;

                // Track backend resolution
                let result_label: &'static str = if result.is_some() { "hit" } else { "miss" };
                monotonic!(
                    layer_cache_backend_resolved = 1,
                    cache_name = self.name.as_str(),
                    backend = BackendType::Postgres.as_ref(),
                    result = result_label
                );

                result
            }

            PersisterMode::S3Primary => {
                // Try S3 first, fallback to PG
                debug!(
                    cache.name = self.name.as_str(),
                    cache.key = key.as_ref(),
                    "attempting S3 read"
                );

                let s3_layers = self
                    .s3_layers
                    .as_ref()
                    .ok_or(LayerDbError::S3NotConfigured)?;

                let s3_layer = s3_layers
                    .get(self.name.as_str())
                    .ok_or(LayerDbError::S3NotConfigured)?;

                debug!(
                    cache.name = self.name.as_str(),
                    cache.key = key.as_ref(),
                    "S3 layer found, calling get"
                );

                match s3_layer.get(key.as_ref()).await? {
                    Some(bytes) => {
                        monotonic!(
                            layer_cache_backend_resolved = 1,
                            cache_name = self.name.as_str(),
                            backend = BackendType::S3.as_ref(),
                            result = "hit"
                        );
                        monotonic!(
                            layer_cache_read_success = 1,
                            cache_name = self.name.as_str(),
                            backend = BackendType::S3.as_ref()
                        );
                        Some(bytes)
                    }
                    None => {
                        debug!(
                            cache.name = self.name.as_str(),
                            cache.key = key.as_ref(),
                            "S3 miss, falling back to PG"
                        );

                        monotonic!(
                            layer_cache_backend_resolved = 1,
                            cache_name = self.name.as_str(),
                            backend = BackendType::S3.as_ref(),
                            result = "miss"
                        );

                        // S3 miss - try PG fallback (keep existing metric calls for now)
                        monotonic!(
                            layer_cache_read_miss = 1,
                            cache_name = self.name.as_str(),
                            backend = BackendType::S3.as_ref()
                        );
                        monotonic!(
                            layer_cache_read_fallback = 1,
                            cache_name = self.name.as_str(),
                            from_backend = BackendType::S3.as_ref(),
                            to_backend = BackendType::Postgres.as_ref()
                        );

                        let result = self.pg.get(&key).await?;

                        let result_label: &'static str =
                            if result.is_some() { "hit" } else { "miss" };
                        monotonic!(
                            layer_cache_backend_resolved = 1,
                            cache_name = self.name.as_str(),
                            backend = BackendType::Postgres.as_ref(),
                            result = result_label
                        );

                        result
                    }
                }
            }

            PersisterMode::S3Only => {
                // Only read from S3
                let s3_layers = self
                    .s3_layers
                    .as_ref()
                    .ok_or(LayerDbError::S3NotConfigured)?;

                let s3_layer = s3_layers
                    .get(self.name.as_str())
                    .ok_or(LayerDbError::S3NotConfigured)?;

                let result = s3_layer.get(key.as_ref()).await?;

                let result_label: &'static str = if result.is_some() { "hit" } else { "miss" };
                monotonic!(
                    layer_cache_backend_resolved = 1,
                    cache_name = self.name.as_str(),
                    backend = BackendType::S3.as_ref(),
                    result = result_label
                );

                result
            }
        };

        // Deserialize and populate cache if found
        match bytes {
            Some(bytes) => {
                debug!(
                    cache.name = self.name.as_str(),
                    cache.key = key.as_ref(),
                    "found in backend, deserializing and caching"
                );

                let deserialized: V = serialize::from_bytes(&bytes)?;

                // Insert into cache for future reads
                self.cache
                    .insert(key.clone(), deserialized.clone(), bytes.len());

                // Emit end-to-end metric for backend hit
                histogram!(
                    layer_cache.request_latency_ms = request_start.elapsed().as_millis() as f64,
                    cache_name = self.name.as_str(),
                    result = "hit"
                );

                Ok(Some(deserialized))
            }
            None => {
                debug!(
                    cache.name = self.name.as_str(),
                    cache.key = key.as_ref(),
                    cache.mode = ?self.mode,
                    "not found in any backend, returning None"
                );

                // Emit end-to-end metric for complete miss
                histogram!(
                    layer_cache.request_latency_ms = request_start.elapsed().as_millis() as f64,
                    cache_name = self.name.as_str(),
                    result = "miss"
                );

                Ok(None)
            }
        }
    }

    #[instrument(
        name = "layer_cache.get_from_memory",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.key = key.as_ref(),
        ),
    )]
    pub async fn get_from_memory(&self, key: Arc<str>) -> LayerDbResult<Option<V>> {
        Ok(self.cache().get_from_memory(key).await)
    }

    #[instrument(
        name = "layer_cache.get_bytes_from_durable_storage",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.key = key.as_ref(),
        ),
    )]
    pub async fn get_bytes_from_durable_storage(
        &self,
        key: Arc<str>,
    ) -> LayerDbResult<Option<Vec<u8>>> {
        debug!(
            cache.name = self.name.as_str(),
            cache.key = key.as_ref(),
            cache.mode = ?self.mode,
            "get_bytes_from_durable_storage called"
        );

        // Fetch from storage backend based on mode - DO NOT use foyer cache
        match self.mode {
            PersisterMode::PostgresOnly | PersisterMode::DualWrite => {
                let result = self.pg.get(&key).await?;

                debug!(
                    cache.name = self.name.as_str(),
                    cache.key = key.as_ref(),
                    found = result.is_some(),
                    backend = "postgres",
                    "get_bytes_from_durable_storage result"
                );

                Ok(result)
            }

            PersisterMode::S3Primary => {
                // Try S3 first, fallback to PG
                let s3_layers = self
                    .s3_layers
                    .as_ref()
                    .ok_or(LayerDbError::S3NotConfigured)?;

                let s3_layer = s3_layers
                    .get(self.name.as_str())
                    .ok_or(LayerDbError::S3NotConfigured)?;

                match s3_layer.get(key.as_ref()).await? {
                    Some(bytes) => {
                        debug!(
                            cache.name = self.name.as_str(),
                            cache.key = key.as_ref(),
                            found = true,
                            backend = "s3",
                            "get_bytes_from_durable_storage result"
                        );
                        Ok(Some(bytes))
                    }
                    None => {
                        debug!(
                            cache.name = self.name.as_str(),
                            cache.key = key.as_ref(),
                            "S3 miss, trying PG fallback"
                        );

                        let result = self.pg.get(&key).await?;

                        debug!(
                            cache.name = self.name.as_str(),
                            cache.key = key.as_ref(),
                            found = result.is_some(),
                            backend = "postgres_fallback",
                            "get_bytes_from_durable_storage result"
                        );

                        Ok(result)
                    }
                }
            }

            PersisterMode::S3Only => {
                // Only read from S3
                let s3_layers = self
                    .s3_layers
                    .as_ref()
                    .ok_or(LayerDbError::S3NotConfigured)?;

                let s3_layer = s3_layers
                    .get(self.name.as_str())
                    .ok_or(LayerDbError::S3NotConfigured)?;

                let result = s3_layer.get(key.as_ref()).await?;

                debug!(
                    cache.name = self.name.as_str(),
                    cache.key = key.as_ref(),
                    found = result.is_some(),
                    backend = "s3",
                    "get_bytes_from_durable_storage result"
                );

                Ok(result)
            }
        }
    }

    pub async fn get_bulk<K>(&self, keys: &[K]) -> LayerDbResult<HashMap<K, V>>
    where
        K: Clone + Display + Eq + Hash + FromStr,
        <K as FromStr>::Err: Display,
    {
        let mut found_keys = HashMap::new();
        let mut not_found: Vec<Arc<str>> = vec![];

        // Check foyer cache first
        for key in keys {
            let key_str: Arc<str> = key.to_string().into();
            if let Some(found) = match self.cache.get(key_str.clone()).await {
                Some(value) => Some(value),
                None => {
                    not_found.push(key_str.clone());
                    None
                }
            } {
                found_keys.insert(key.clone(), found);
            }
        }

        // Fetch missing keys from backend based on mode
        if !not_found.is_empty() {
            let backend_found = match self.mode {
                PersisterMode::PostgresOnly | PersisterMode::DualWrite => {
                    self.pg.get_many(&not_found).await?
                }

                PersisterMode::S3Primary => {
                    // Try S3 first
                    let s3_layers = self
                        .s3_layers
                        .as_ref()
                        .ok_or(LayerDbError::S3NotConfigured)?;

                    let s3_layer = s3_layers
                        .get(self.name.as_str())
                        .ok_or(LayerDbError::S3NotConfigured)?;

                    // Convert Vec<Arc<str>> to Vec<&str>
                    let keys_refs: Vec<&str> = not_found.iter().map(|k| k.as_ref()).collect();
                    let s3_results = s3_layer.get_bulk(&keys_refs).await?;

                    if !s3_results.is_empty() {
                        // Find keys not in S3 for PG fallback
                        let still_not_found: Vec<Arc<str>> = not_found
                            .iter()
                            .filter(|k| !s3_results.contains_key(k.as_ref()))
                            .cloned()
                            .collect();

                        if !still_not_found.is_empty() {
                            // Try PG fallback for remaining keys
                            if let Some(pg_results) = self.pg.get_many(&still_not_found).await? {
                                // Merge S3 and PG results
                                let mut combined = s3_results;
                                combined.extend(pg_results);
                                Some(combined)
                            } else {
                                Some(s3_results)
                            }
                        } else {
                            Some(s3_results)
                        }
                    } else {
                        // All keys missed S3, try PG fallback
                        self.pg.get_many(&not_found).await?
                    }
                }

                PersisterMode::S3Only => {
                    let s3_layers = self
                        .s3_layers
                        .as_ref()
                        .ok_or(LayerDbError::S3NotConfigured)?;

                    let s3_layer = s3_layers
                        .get(self.name.as_str())
                        .ok_or(LayerDbError::S3NotConfigured)?;

                    // Convert Vec<Arc<str>> to Vec<&str>
                    let keys_refs: Vec<&str> = not_found.iter().map(|k| k.as_ref()).collect();
                    let results = s3_layer.get_bulk(&keys_refs).await?;
                    if results.is_empty() {
                        None
                    } else {
                        Some(results)
                    }
                }
            };

            if let Some(results) = backend_found {
                for (k, bytes) in results {
                    let deserialized: V = serialize::from_bytes(&bytes)?;
                    self.cache
                        .insert(k.clone().into(), deserialized.clone(), bytes.len());
                    found_keys.insert(
                        K::from_str(&k).map_err(|err| {
                            LayerDbError::CouldNotConvertToKeyFromString(err.to_string())
                        })?,
                        deserialized,
                    );
                }
            }
        }

        Ok(found_keys)
    }

    pub async fn deserialize_memory_value(&self, bytes: Arc<Vec<u8>>) -> LayerDbResult<V> {
        serialize::from_bytes_async(&bytes).await
    }

    pub fn cache(&self) -> Cache<V> {
        self.cache.clone()
    }

    pub fn pg(&self) -> PgLayer {
        self.pg.clone()
    }

    pub fn remove_from_memory(&self, key: &str) {
        self.cache.remove(key);
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains(key)
    }

    pub fn insert(&self, key: Arc<str>, value: V, size_hint: usize) {
        if !self.cache.contains(&key) {
            self.cache.insert(key, value, size_hint);
        }
    }

    pub fn insert_from_cache_updates(&self, key: Arc<str>, serialize_value: Vec<u8>) {
        self.cache
            .insert_raw_bytes(key.clone(), serialize_value.clone());
    }

    pub fn insert_or_update(&self, key: Arc<str>, value: V, size_hint: usize) {
        self.cache.insert(key, value, size_hint);
    }

    pub fn insert_or_update_from_cache_updates(&self, key: Arc<str>, serialize_value: Vec<u8>) {
        self.insert_from_cache_updates(key, serialize_value)
    }

    pub fn evict_from_cache_updates(&self, key: Arc<str>) {
        self.cache.remove(&key);
    }
}
