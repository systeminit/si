//! A fast in-memory, network aware, layered write-through cache for System Initiative.
//!
//! # Architecture
//!
//! The cache has 3 layers:
//!
//! * **Foyer** - In-memory LRU cache with optional disk backing
//! * **S3** - Object storage persistence with internal write queue and adaptive rate limiting
//! * **Postgres** - Legacy persistence layer (being phased out)
//!
//! ## Write Path
//!
//! When a write is requested:
//!
//! 1. Data written to Foyer in-memory cache
//! 2. Foyer handles disk shuffling when appropriate
//! 3. Data published to NATS topic for remote cache population
//! 4. **S3 writes enqueued to persistent disk queue** (no fast path for durability)
//! 5. Background processor dequeues and writes to S3 with adaptive rate limiting
//! 6. Postgres written to by persister process (dual-write mode, legacy)
//!
//! ## S3 Write Queue
//!
//! All S3 writes go through a persistent queue for durability guarantees:
//!
//! - **Queue directory:** `{base_path}/{cache_name}_s3_queue/`
//! - **File format:** `{ULID}.pending` (Postcard-serialized `LayeredEvent`)
//! - **Ordering:** ULID-based chronological order
//! - **Dead letter queue:** `dead_letter/` subdirectory for corrupted data
//!
//! See [`s3_write_queue`] and [`s3_queue_processor`] modules for details.
//!
//! ## Adaptive Rate Limiting
//!
//! S3 writes are rate-limited to avoid constant throttling:
//!
//! - **Exponential backoff** on throttling (503, SlowDown, etc.)
//! - **Gradual reduction** after consecutive successes
//! - **Configurable parameters** via [`RateLimitConfig`]
//!
//! See [`rate_limiter`] module for state machine details.
//!
//! ## Read Path
//!
//! When a read is requested:
//!
//! 1. Data read from Foyer (in-memory or disk)
//! 2. On miss, read from S3
//! 3. On miss, read from Postgres (legacy)
//! 4. Populate Foyer with fetched data
//!
//! ## Retry Queue (Legacy)
//!
//! For transient PostgreSQL failures, writes are persisted to a retry queue with exponential
//! backoff. This will be removed when PostgreSQL persistence is phased out.
//!
//! See [`retry_queue`] module for details.
//!
#![allow(clippy::doc_lazy_continuation)]

use serde::{
    Deserialize,
    Serialize,
};
use strum::AsRefStr;

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
pub mod rate_limiter;
pub mod retry_queue;
pub mod s3;
pub mod s3_queue_processor;
pub mod s3_write_queue;

#[derive(AsRefStr, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum BackendType {
    Postgres,
    S3,
}

pub use db::LayerDb;
pub use error::LayerDbError;
pub use persister::PersisterMode;
pub use pg::{
    APPLICATION_NAME,
    DBNAME,
    default_pg_pool_config,
};
pub use rate_limiter::{
    RateLimitConfig,
    RateLimitConfigError,
    RateLimiter,
};
pub use s3::{
    KeyTransformStrategy,
    ObjectStorageConfig,
    S3AuthConfig,
    S3Layer,
};
