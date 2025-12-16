//! A fast in-memory, network aware, layered write-through cache for System Initiative.
//!
//! # Architecture
//!
//! The cache has 3 layers:
//!
//! * **Foyer** - In-memory LRU cache with optional disk backing
//! * **S3** - Object storage persistence with internal write queue and adaptive rate limiting
//! * **Postgres** - Database persistence layer
//!
//! ## Write Path
//!
//! Write behavior depends on the configured [`PersisterMode`]:
//!
//! 1. Data written to Foyer in-memory cache
//! 2. Foyer handles disk shuffling when appropriate
//! 3. Data published to NATS topic for remote cache population
//! 4. Persistence backends (S3, Postgres, or both) handle durable storage based on mode
//!
//! For S3-enabled modes (`DualWrite`, `S3Primary`, `S3Only`):
//! - All S3 writes are enqueued to persistent disk queue (no fast path for durability)
//! - Background processor dequeues and writes to S3 with adaptive rate limiting
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
//! See [`s3_disk_store`] and [`s3_queue_processor`] modules for details.
//!
//! ## S3 Throttling and Retries
//!
//! S3 throttling is handled by the AWS SDK's built-in retry mechanism:
//!
//! - **SDK Standard Retry Mode** with exponential backoff for throttling errors
//! - **Multi-worker architecture** distributes load across concurrent workers
//! - **Worker count** configurable via `S3QueueProcessorConfig::worker_count`
//!
//! ## Read Path
//!
//! Read behavior depends on the configured [`PersisterMode`]:
//!
//! - **PostgresOnly, DualWrite:** Read from Foyer → Postgres
//! - **S3Primary:** Read from Foyer → S3 → Postgres (fallback)
//! - **S3Only:** Read from Foyer → S3
//!
//! Fetched data populates Foyer for future reads.
//!
//! ## Retry Queue
//!
//! For transient PostgreSQL failures, writes are persisted to a retry queue with exponential
//! backoff.
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
pub mod retry_queue;
pub mod s3;
pub mod s3_disk_store;
pub mod s3_queue_processor;
pub mod s3_worker;

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
pub use s3::{
    KeyTransformStrategy,
    ObjectStorageConfig,
    S3AuthConfig,
    S3Layer,
    S3ReadRetryConfig,
};
