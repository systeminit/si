//! A fast in-memory, network aware, layered write-through cache for System Initiative.
//!
//! It should have 3 layers of caching:
//!
//! * Moka, an in-memory LRU style cache.
//! * Cacache, an on-disk to keep more data locally than can be held in memory
//! * Postgres, our final persistant storage layer.
//!
//! When a write is requested, the following happens:
//!
//! * The data is written first to a Moka cache
//! * Then written to cacache for persistent storage
//! * The data is then published to a nats topic layer-cache.workspaceId
//! * Any remote si-layer-cache instances listen to this topic, and populate their local caches
//! * Postgres gets written to eventually by a 'persister' process that writes to PG from the write
//! stream
//!
//! When a read is requested, the following happen:
//!
//! * The data is read from the moka cache
//! * On a miss, the data is read from cacache, inserted into Moka, and returned to the user
//! * On a miss, the data is read from Postgres, inserted into sled, inserted into moka, and
//! returned to the user
//!

pub mod activities;
mod activity_client;
pub mod db;
pub mod disk_cache;
pub mod error;
pub mod event;
pub mod layer_cache;
pub mod memory_cache;
mod nats;
pub mod persister;
pub mod pg;

pub use db::LayerDb;
pub use disk_cache::{default_cacache_path, default_cache_path_for_service, CaCacheTempFile};
pub use error::LayerDbError;
