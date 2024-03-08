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

pub mod chunking_nats;
pub mod disk_cache;
pub mod error;
pub mod memory_cache;
pub mod pg;

use serde::{de::DeserializeOwned, Serialize};
use si_data_pg::{PgPool, PgPoolConfig};
use std::{
    hash::Hash,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::task::JoinHandle;

pub use disk_cache::default_sled_path;
use disk_cache::DiskCache;
use error::LayerCacheResult;
use memory_cache::MemoryCache;
use pg::PgLayer;
pub use pg::{default_pg_pool_config, APPLICATION_NAME, DBNAME};

#[derive(Clone)]
pub struct LayerCache<K, V>
where
    K: AsRef<[u8]> + Eq + Hash + Copy + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    memory_cache: MemoryCache<K, V>,
    disk_cache: Arc<DiskCache<K>>,
    pg: PgLayer<K>,
    active_disk_writes: Arc<AtomicU64>,
}

impl<K, V> LayerCache<K, V>
where
    K: AsRef<[u8]> + Eq + Hash + Copy + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn new(name: &str, fast_disk: sled::Db, pg_pool: PgPool) -> LayerCacheResult<Self> {
        let disk_cache = Arc::new(DiskCache::new(fast_disk, name.as_bytes())?);

        let pg = PgLayer::new(pg_pool);
        pg.migrate().await?;

        Ok(LayerCache {
            memory_cache: MemoryCache::new(),
            disk_cache,
            pg,
            active_disk_writes: Arc::new(AtomicU64::new(0)),
        })
    }

    async fn inc_writes(&self) {
        let mut current = self.active_disk_writes.load(Ordering::Relaxed);
        loop {
            // Panic if we have u64::MAX concurrent writes
            assert!(current < u64::MAX, "write counter overflow in LayerCache");
            match self.active_disk_writes.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(raced_value) => current = raced_value,
            }
            tokio::task::yield_now().await;
        }
    }

    async fn dec_writes(&self) {
        let mut current = self.active_disk_writes.load(Ordering::Relaxed);
        loop {
            if current == 0 {
                break;
            }
            match self.active_disk_writes.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(raced_value) => current = raced_value,
            }
            tokio::task::yield_now().await;
        }
    }

    async fn spawn_disk_cache_write(&self, key: K, value: V) {
        self.inc_writes().await;
        let self_clone = self.clone();
        let write_handle = tokio::task::spawn(async move {
            if let Ok(serialized) = postcard::to_stdvec(&value) {
                let _ = self_clone.disk_cache.insert(key, &serialized);
            }
        });

        self.handle_write_finish(write_handle).await;
    }

    async fn spawn_disk_cache_write_vec(&self, key: K, value: Vec<u8>) {
        self.inc_writes().await;
        let self_clone = self.clone();
        let write_handle = tokio::task::spawn(async move {
            let _ = self_clone.disk_cache.insert(key, &value);
        });

        self.handle_write_finish(write_handle).await;
    }

    async fn spawn_pg_write(&self, key: K, value: V) {
        self.inc_writes().await;
        let self_clone = self.clone();
        let write_handle = tokio::task::spawn(async move {
            if let Ok(serialized) = postcard::to_stdvec(&value) {
                let _ = self_clone.pg.insert(key, &serialized).await;
            }
        });

        self.handle_write_finish(write_handle).await;
    }

    async fn handle_write_finish(&self, join_handle: JoinHandle<()>) {
        let self_clone = self.clone();
        tokio::task::spawn(async move {
            let _ = join_handle.await;
            self_clone.dec_writes().await;
        });
    }

    pub async fn get(&self, key: &K) -> LayerCacheResult<Option<V>> {
        Ok(match self.memory_cache.get(key).await {
            Some(memory_value) => Some(memory_value),
            None => match self.disk_cache.get(key)? {
                Some(value) => {
                    let deserialized: V = postcard::from_bytes(&value)?;

                    self.memory_cache.insert(*key, deserialized.clone()).await;
                    Some(deserialized)
                }
                None => match self.pg.get(key).await? {
                    Some(value) => {
                        let deserialized: V = postcard::from_bytes(&value)?;

                        self.memory_cache.insert(*key, deserialized.clone()).await;
                        self.spawn_disk_cache_write_vec(*key, value).await;

                        Some(deserialized)
                    }
                    None => None,
                },
            },
        })
    }

    pub fn memory_cache(&self) -> MemoryCache<K, V> {
        self.memory_cache.clone()
    }

    pub fn disk_cache(&self) -> Arc<DiskCache<K>> {
        self.disk_cache.clone()
    }

    pub fn pg(&self) -> PgLayer<K> {
        self.pg.clone()
    }

    pub async fn remove_from_memory(&self, key: K) {
        self.memory_cache.remove(&key).await;
    }

    /// The disk and database writers will spawn a thread to perform the write,
    /// and that thread must be joined on if all the writes are to succeed (the
    /// caller may terminate before the write threads). This method will block
    /// until all writes have succeeded.
    pub async fn join_all_write_tasks(&self) {
        while self.active_disk_writes.load(Ordering::Relaxed) > 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    pub async fn insert(&self, key: K, value: V) -> LayerCacheResult<()> {
        let in_memory = self.memory_cache.contains(&key);
        let on_disk = self.disk_cache.contains_key(&key)?;

        match (in_memory, on_disk) {
            // In memory and on disk, do nothing
            (true, true) => (),
            // Neither on memory or on disk
            (false, false) => {
                self.memory_cache.insert(key, value.clone()).await;
                self.spawn_disk_cache_write(key, value.clone()).await;
            }
            // Not in memory, but on disk - we can write, because objects are immutable
            (false, true) => {
                self.memory_cache.insert(key, value.clone()).await;
            }
            // In memory, but not on disk
            (true, false) => {
                self.spawn_disk_cache_write(key, value.clone()).await;
            }
        }

        self.spawn_pg_write(key, value).await;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct LayerCacheDependencies {
    pub sled: sled::Db,
    pub pg_pool: PgPool,
}

pub async fn make_layer_cache_dependencies<P: AsRef<Path>>(
    sled_path: P,
    pg_pool_config: &PgPoolConfig,
) -> LayerCacheResult<LayerCacheDependencies> {
    Ok(LayerCacheDependencies {
        sled: sled::open(sled_path)?,
        pg_pool: PgPool::new(pg_pool_config).await?,
    })
}
