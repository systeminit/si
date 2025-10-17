//! Persistent retry queue for handling transient PostgreSQL write failures.
//!
//! # Overview
//!
//! The retry queue system ensures data durability by persisting failed writes to disk
//! and retrying them with exponential backoff. Each LayerCache gets its own retry queue
//! directory with individual files per failed write.
//!
//! # Architecture
//!
//! - **RetryQueueManager**: Central component managing all retry queues
//! - **QueueState**: Per-cache backoff and file tracking
//! - **File format**: Postcard-serialized `LayeredEvent` with ULID filenames for ordering
//! - **Backoff**: Exponential (100ms → 5s max), resets on success per queue
//!
//! # Directory Structure
//!
//! ```text
//! {base_path}/
//! ├── cas_retries/
//! │   ├── 01JQRS123456789ABCDEFG.pending
//! │   └── 01JQRS234567890BCDEFGH.pending
//! ├── workspace_snapshot_retries/
//! │   └── 01JQRS345678901CDEFGHI.pending
//! └── ...
//! ```
//!
//! # Configuration
//!
//! Configure via `RetryQueueConfig`:
//! - `base_path`: Directory for all retry queues (default: alongside disk cache)
//! - `initial_backoff`: Starting delay (default: 100ms)
//! - `max_backoff`: Maximum delay (default: 5s)
//! - `backoff_multiplier`: Growth factor (default: 2.0)

use std::{
    collections::{
        BTreeSet,
        HashMap,
    },
    ffi::OsString,
    path::PathBuf,
    time::{
        Duration,
        Instant,
    },
};

use crate::{
    error::{
        LayerDbError,
        LayerDbResult,
    },
    event::LayeredEvent,
};

/// Configuration for retry queue behavior
#[derive(Debug, Clone)]
pub struct RetryQueueConfig {
    /// Base directory for all retry queues
    pub base_path: PathBuf,
    /// Initial backoff duration (default: 100ms)
    pub initial_backoff: Duration,
    /// Maximum backoff duration (default: 5 seconds)
    pub max_backoff: Duration,
    /// Backoff multiplier (default: 2.0)
    pub backoff_multiplier: f64,
}

impl Default for RetryQueueConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("retry_queues"),
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

/// Tracks backoff state for a single queue
#[derive(Debug)]
struct QueueState {
    retry_dir: PathBuf,
    current_backoff: Duration,
    next_retry_time: Instant,
    pending_files: BTreeSet<OsString>,
}

impl QueueState {
    fn new(retry_dir: PathBuf, initial_backoff: Duration) -> Self {
        Self {
            retry_dir,
            current_backoff: initial_backoff,
            next_retry_time: Instant::now(),
            pending_files: BTreeSet::new(),
        }
    }

    /// Record a failed retry attempt and update backoff
    fn record_failure(
        &mut self,
        max_backoff: Duration,
        multiplier: f64,
        initial_backoff: Duration,
    ) {
        self.current_backoff = calculate_next_backoff(
            self.current_backoff,
            max_backoff,
            multiplier,
            initial_backoff,
        );
        self.next_retry_time = Instant::now() + self.current_backoff;
    }

    /// Record a successful retry and reset backoff
    fn record_success(&mut self, initial_backoff: Duration) {
        self.current_backoff = initial_backoff;
        self.next_retry_time = Instant::now();
    }

    /// Check if this queue is ready to retry
    fn is_ready(&self) -> bool {
        Instant::now() >= self.next_retry_time && !self.pending_files.is_empty()
    }
}

/// Handle for tracking a retry attempt
#[derive(Debug)]
pub struct RetryHandle {
    pub(crate) cache_name: String,
    pub(crate) filename: OsString,
}

/// Calculate the next backoff duration using exponential backoff with full jitter
///
/// Full jitter prevents thundering herd by randomizing retry times between the
/// minimum (initial_backoff) and maximum (calculated exponential backoff capped at max).
/// This follows the AWS recommendation and matches the pattern in dal/job/consumer.rs.
fn calculate_next_backoff(
    current: Duration,
    max: Duration,
    multiplier: f64,
    initial: Duration,
) -> Duration {
    use rand::Rng;

    // Calculate next exponential backoff value, clamping to max
    let next_secs = current.as_secs_f64() * multiplier;
    let next_backoff = std::cmp::min(Duration::from_secs_f64(next_secs), max);

    // Apply full jitter: random value between initial_backoff and calculated next_backoff
    let mut rng = rand::thread_rng();
    let min_micros = initial.as_micros() as u64;
    let max_micros = next_backoff.as_micros() as u64;
    let jittered_micros = rng.gen_range(min_micros..=max_micros);

    Duration::from_micros(jittered_micros)
}

/// Determine if a LayerDbError represents a transient failure that should be retried
pub fn is_retryable_error(error: &LayerDbError) -> bool {
    match error {
        // Direct PG errors are always retryable (connection/pool issues)
        LayerDbError::Pg(_) | LayerDbError::PgPool(_) => true,

        // For PersisterTaskFailed, only retry if PostgreSQL failed
        // NATS-only failures are not retried - remote services will fetch from PG on cache miss
        LayerDbError::PersisterTaskFailed(task_error) => task_error.pg_error.is_some(),

        // All other errors are non-retryable
        _ => false,
    }
}

/// Write a LayeredEvent to a retry queue file with fsync for durability
///
/// Uses write-to-temp-then-rename for atomicity, with fsync before rename
/// to ensure data is on disk even if system crashes immediately after write.
async fn write_retry_file(path: &std::path::Path, event: &LayeredEvent) -> LayerDbResult<()> {
    use tokio::{
        fs,
        io::AsyncWriteExt,
    };

    let bytes = postcard::to_stdvec(event)?;

    // Write to temp file first, then atomic rename
    let temp_path = path.with_extension("tmp");

    // Open file for writing
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_path)
        .await
        .map_err(LayerDbError::RetryQueueFileWrite)?;

    // Write data to file
    file.write_all(&bytes)
        .await
        .map_err(LayerDbError::RetryQueueFileWrite)?;

    // Flush to disk - this is the critical durability guarantee
    file.sync_all()
        .await
        .map_err(LayerDbError::RetryQueueFileWrite)?;

    // Explicitly close file before rename (drop)
    drop(file);

    // Atomic rename to final location
    fs::rename(&temp_path, path)
        .await
        .map_err(LayerDbError::RetryQueueFileWrite)?;

    Ok(())
}

/// Read a LayeredEvent from a retry queue file
async fn read_retry_file(path: &std::path::Path) -> LayerDbResult<LayeredEvent> {
    use tokio::fs;

    let bytes = fs::read(path)
        .await
        .map_err(LayerDbError::RetryQueueFileRead)?;

    let event = postcard::from_bytes(&bytes)?;

    Ok(event)
}

/// Delete a retry queue file
async fn delete_retry_file(path: &std::path::Path) -> LayerDbResult<()> {
    use tokio::fs;

    fs::remove_file(path)
        .await
        .map_err(LayerDbError::RetryQueueFileDelete)?;

    Ok(())
}

/// Manages persistent retry queues for all LayerCache instances
#[derive(Debug)]
pub struct RetryQueueManager {
    config: RetryQueueConfig,
    queues: HashMap<String, QueueState>,
}

impl RetryQueueManager {
    pub fn new(config: RetryQueueConfig) -> Self {
        Self {
            config,
            queues: HashMap::new(),
        }
    }
}
