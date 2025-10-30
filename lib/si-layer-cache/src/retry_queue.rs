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
//! # Usage
//!
//! The retry queue is integrated into `PersisterTask`:
//!
//! 1. New writes attempt direct PostgreSQL write
//! 2. On transient failure (connection/network), event is enqueued to disk
//! 3. Background loop continuously attempts retries with backoff
//! 4. On success, file is deleted and backoff resets
//! 5. On startup, existing queues are scanned and retried immediately
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

use telemetry_utils::{
    gauge,
    monotonic,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

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

/// Messages sent to RetryQueueManager for queue operations
#[derive(Debug)]
pub enum RetryQueueMessage {
    /// Enqueue a failed event for retry
    Enqueue(LayeredEvent),
    /// Mark a retry attempt as successful (removes from queue)
    MarkSuccess(RetryHandle),
    /// Mark a retry attempt as failed with retryable error (updates backoff)
    MarkRetryableFailure(RetryHandle, LayerDbError),
    /// Mark a retry attempt as permanently failed (removes from queue)
    MarkPermanentFailure(RetryHandle),
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

    /// Run the retry queue manager as an independent task
    pub async fn run(
        mut self,
        mut queue_rx: mpsc::UnboundedReceiver<RetryQueueMessage>,
        ready_tx: mpsc::UnboundedSender<(LayeredEvent, RetryHandle)>,
        shutdown_token: CancellationToken,
    ) {
        use telemetry::prelude::*;

        loop {
            tokio::select! {
                biased;

                _ = shutdown_token.cancelled() => {
                    debug!("RetryQueueManager received shutdown signal");
                    break;
                }

                Some(msg) = queue_rx.recv() => {
                    self.handle_message(msg).await;
                }

                result = self.wait_for_ready_retry() => {
                    let _ = ready_tx.send(result);
                }
            }
        }

        debug!("RetryQueueManager shutdown complete");
    }

    /// Handle incoming messages from PersisterTask and spawned tasks
    async fn handle_message(&mut self, msg: RetryQueueMessage) {
        use telemetry::prelude::*;

        match msg {
            RetryQueueMessage::Enqueue(event) => {
                if let Err(err) = self.enqueue(event).await {
                    error!(error = ?err, "failed to enqueue for retry");
                }
            }
            RetryQueueMessage::MarkSuccess(handle) => {
                if let Err(err) = self.mark_success(handle).await {
                    error!(error = ?err, "failed to mark retry as successful");
                }
            }
            RetryQueueMessage::MarkRetryableFailure(handle, error) => {
                self.mark_failure(handle, &error);
            }
            RetryQueueMessage::MarkPermanentFailure(handle) => {
                self.mark_permanent_failure(handle).await;
            }
        }
    }

    /// Scan filesystem for existing retry queue directories and load pending files
    pub async fn scan_existing_queues(&mut self, cache_names: &[&str]) -> LayerDbResult<()> {
        use telemetry::prelude::*;
        use tokio::fs;

        for &cache_name in cache_names {
            let retry_dir = self.config.base_path.join(format!("{cache_name}_retries"));

            // Skip if directory doesn't exist
            if !retry_dir.exists() {
                continue;
            }

            // Read directory entries
            let mut entries = fs::read_dir(&retry_dir)
                .await
                .map_err(LayerDbError::RetryQueueDirRead)?;

            let mut pending_files = BTreeSet::new();

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(LayerDbError::RetryQueueDirRead)?
            {
                let path = entry.path();

                // Only process .pending files
                if path.extension() == Some(std::ffi::OsStr::new("pending")) {
                    pending_files.insert(entry.file_name());
                }
            }

            if !pending_files.is_empty() {
                info!(
                    cache.name = cache_name,
                    queue.depth = pending_files.len(),
                    "found existing retry queue on startup"
                );

                gauge!(
                    layer_cache_retry_queue_depth = pending_files.len(),
                    cache_name = cache_name
                );

                let mut state = QueueState::new(retry_dir, self.config.initial_backoff);
                state.pending_files = pending_files;
                // Retry immediately on startup
                state.next_retry_time = Instant::now();

                self.queues.insert(cache_name.to_string(), state);
            }
        }

        Ok(())
    }

    /// Enqueue a failed write for retry
    pub async fn enqueue(&mut self, event: LayeredEvent) -> LayerDbResult<()> {
        use telemetry::prelude::*;
        use tokio::fs;
        use ulid::Ulid;

        let cache_name = event.payload.db_name.to_string();
        let retry_dir = self.config.base_path.join(format!("{cache_name}_retries"));

        // Ensure retry directory exists
        fs::create_dir_all(&retry_dir)
            .await
            .map_err(LayerDbError::RetryQueueDirCreate)?;

        // Generate ULID for ordering and uniqueness
        let ulid = Ulid::new();
        let filename = format!("{ulid}.pending");
        let file_path = retry_dir.join(&filename);

        // Write event to disk
        write_retry_file(&file_path, &event).await?;

        // Get or create queue state
        let state = self
            .queues
            .entry(cache_name.clone())
            .or_insert_with(|| QueueState::new(retry_dir, self.config.initial_backoff));

        // Add filename to pending set
        state.pending_files.insert(filename.into());

        let queue_depth = state.pending_files.len();

        info!(
            cache.name = %cache_name,
            event.kind = ?event.event_kind,
            event.ulid = %ulid,
            queue.depth = queue_depth,
            "persister write failed, enqueued for retry"
        );

        monotonic!(
            layer_cache_retry_queue_enqueued = 1,
            cache_name = cache_name
        );
        gauge!(
            layer_cache_retry_queue_depth = queue_depth,
            cache_name = cache_name
        );

        Ok(())
    }

    /// Check if an error should result in enqueueing for retry
    pub fn should_enqueue(&self, error: &LayerDbError) -> bool {
        is_retryable_error(error)
    }

    /// Get the next retry time for a specific cache queue (for testing/observability)
    pub fn next_retry_time(&self, cache_name: &str) -> Option<Instant> {
        self.queues
            .get(cache_name)
            .map(|state| state.next_retry_time)
    }

    /// Try to get a ready retry without blocking
    async fn try_get_ready_retry(&mut self) -> Option<(LayeredEvent, RetryHandle)> {
        use telemetry::prelude::*;

        // Find first ready queue with pending files
        let cache_name = self
            .queues
            .iter()
            .find(|(_, state)| state.is_ready())
            .map(|(name, _)| name.clone())?;

        let state = self.queues.get_mut(&cache_name)?;

        // Get first filename (BTreeSet is sorted)
        let filename = state.pending_files.iter().next()?.clone();
        let file_path = state.retry_dir.join(&filename);

        // Read event from disk
        match read_retry_file(&file_path).await {
            Ok(event) => {
                debug!(
                    cache.name = %cache_name,
                    filename = ?filename,
                    backoff_ms = state.current_backoff.as_millis(),
                    "attempting retry from queue"
                );

                // Remove from pending files immediately to prevent duplicate retries
                state.pending_files.remove(&filename);

                let handle = RetryHandle {
                    cache_name: cache_name.clone(),
                    filename,
                };

                Some((event, handle))
            }
            Err(err) => {
                error!(
                    cache.name = %cache_name,
                    filename = ?filename,
                    error = %err,
                    "failed to read retry file, skipping"
                );

                // Remove corrupted file from tracking
                state.pending_files.remove(&filename);
                None
            }
        }
    }

    /// Calculate how long to sleep until the next queue becomes ready
    fn calculate_next_wakeup(&self) -> Duration {
        self.queues
            .values()
            .filter(|q| !q.pending_files.is_empty())
            .map(|q| q.next_retry_time.saturating_duration_since(Instant::now()))
            .min()
            .unwrap_or(Duration::from_secs(60)) // Default: check every minute if no queues
    }

    /// Waits until at least one retry is ready, then returns it
    async fn wait_for_ready_retry(&mut self) -> (LayeredEvent, RetryHandle) {
        loop {
            // Try to get a ready retry immediately
            if let Some(result) = self.try_get_ready_retry().await {
                return result;
            }

            // Calculate sleep until next retry becomes ready
            let sleep_duration = self.calculate_next_wakeup();
            tokio::time::sleep(sleep_duration).await;
        }
    }

    /// Mark a retry attempt as successful and remove from queue
    pub async fn mark_success(&mut self, handle: RetryHandle) -> LayerDbResult<()> {
        use telemetry::prelude::*;

        let state = match self.queues.get_mut(&handle.cache_name) {
            Some(state) => state,
            None => return Ok(()), // Queue was removed, nothing to do
        };

        let file_path = state.retry_dir.join(&handle.filename);

        // Delete file from disk
        if let Err(err) = delete_retry_file(&file_path).await {
            warn!(
                cache.name = %handle.cache_name,
                filename = ?handle.filename,
                error = %err,
                "failed to delete retry file after success"
            );
        }

        // Note: file was already removed from pending_files when it was retrieved for retry

        // Reset backoff on success
        state.record_success(self.config.initial_backoff);

        let queue_depth = state.pending_files.len();

        info!(
            cache.name = %handle.cache_name,
            filename = ?handle.filename,
            queue.depth = queue_depth,
            "retry successful, removed from queue"
        );

        monotonic!(
            layer_cache_retry_queue_success = 1,
            cache_name = &handle.cache_name
        );
        gauge!(
            layer_cache_retry_queue_depth = queue_depth,
            cache_name = &handle.cache_name
        );

        Ok(())
    }

    /// Mark a retry attempt as failed and update backoff
    pub fn mark_failure(&mut self, handle: RetryHandle, error: &LayerDbError) {
        use telemetry::prelude::*;

        let state = match self.queues.get_mut(&handle.cache_name) {
            Some(state) => state,
            None => return, // Queue was removed, nothing to do
        };

        // Re-add file to pending set since retry failed
        state.pending_files.insert(handle.filename.clone());

        // Update backoff
        state.record_failure(
            self.config.max_backoff,
            self.config.backoff_multiplier,
            self.config.initial_backoff,
        );

        let error_type = format!("{error:?}")
            .split('(')
            .next()
            .unwrap_or("Unknown")
            .to_string();

        warn!(
            cache.name = %handle.cache_name,
            filename = ?handle.filename,
            retry.backoff_ms = state.current_backoff.as_millis(),
            error = %error,
            error.type = %error_type,
            "retry failed, will retry after backoff"
        );

        let queue_depth = state.pending_files.len();

        monotonic!(
            layer_cache_retry_queue_failed = 1,
            cache_name = &handle.cache_name
        );
        gauge!(
            layer_cache_retry_queue_depth = queue_depth,
            cache_name = &handle.cache_name
        );
    }

    /// Mark a retry attempt as permanently failed and remove from queue
    pub async fn mark_permanent_failure(&mut self, handle: RetryHandle) {
        use telemetry::prelude::*;

        let state = match self.queues.get_mut(&handle.cache_name) {
            Some(state) => state,
            None => return, // Queue was removed, nothing to do
        };

        let file_path = state.retry_dir.join(&handle.filename);

        // Delete file from disk
        if let Err(err) = delete_retry_file(&file_path).await {
            warn!(
                cache.name = %handle.cache_name,
                filename = ?handle.filename,
                error = %err,
                "failed to delete retry file after permanent failure"
            );
        }

        // Note: file was already removed from pending_files when it was retrieved for retry

        let queue_depth = state.pending_files.len();

        warn!(
            cache.name = %handle.cache_name,
            filename = ?handle.filename,
            queue.depth = queue_depth,
            "retry permanently failed (non-retryable error), removed from queue"
        );

        monotonic!(
            layer_cache_retry_queue_permanent_failure = 1,
            cache_name = &handle.cache_name
        );
        gauge!(
            layer_cache_retry_queue_depth = queue_depth,
            cache_name = &handle.cache_name
        );
    }
}
