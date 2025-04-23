//! A fast in-memory, network aware, layered write-through cache for System Initiative.
//!
//! It should have 3 layers of caching:
//!
//! * Foyer, an in-memory LRU style cache.
//! * Foyer, which also include an on-disk to keep more data locally than can be held in memory.
//! * Postgres, our final persistant storage layer.
//!
//! When a write is requested, the following happens:
//!
//! * The data is written first to Foyer in-memory
//! * Foyer handles shuffling to the disk when appropriate
//! * The data is then published to a nats topic layer-cache.workspaceId
//! * Any remote si-layer-cache instances listen to this topic, and populate their local caches
//! * Postgres gets written to eventually by a 'persister' process that writes to PG from the write
//! stream
//!
//! When a read is requested, the following happen:
//!
//! * The data is read from foyer
//! * On a miss in-memory, Foyer gets it from disk, promotes it to in-memory, and returns it to the user
//! * On a miss, the data is read from Postgres, and then inserted in Foyer
//! returned to the user
//!
#![allow(clippy::doc_lazy_continuation)]

pub mod activities;
mod activity_client;
pub mod db;
pub mod error;
pub mod event;
pub mod hybrid_cache;
pub mod layer_cache;
mod nats;
pub mod persister;
pub mod pg;

pub use db::LayerDb;
pub use error::LayerDbError;
pub use pg::{
    APPLICATION_NAME,
    DBNAME,
    default_pg_pool_config,
};

// FIXME(nick): this needs to move outside the layer cache.
pub mod external {
    pub mod rebaser_server {
        pub use crate::nats::{
            rebaser_requests_work_queue_stream,
            subject::for_rebaser_requests,
        };
    }
}
