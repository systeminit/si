use std::sync::{
    Arc,
    atomic::{
        AtomicUsize,
        Ordering,
    },
};

use aws_sdk_s3::Client;
use crossbeam_queue::SegQueue;
use telemetry::prelude::*;
use telemetry_utils::{
    histogram,
    monotonic,
};
use tokio::{
    sync::Notify,
    task::JoinSet,
};

use crate::{
    s3_disk_store::{
        S3DiskStore,
        S3DiskStoreError,
    },
    s3_queue_processor::WorkItem,
};

/// Result of an upload attempt
#[derive(Debug)]
pub struct UploadResult {
    pub work_item: WorkItem,
    pub outcome: UploadOutcome,
}

/// Reason for moving to dead letter queue
#[derive(Debug, thiserror::Error)]
pub enum DeadLetterQueueReason {
    #[error("Failed to read event from disk")]
    DiskReadError(#[source] S3DiskStoreError),

    #[error("S3 serialization error: {0}")]
    SerializationError(String),
}

/// Outcome of an upload attempt
#[derive(Debug)]
pub enum UploadOutcome {
    /// Upload succeeded, delete from disk
    Success,
    /// SDK exhausted retries, re-enqueue for application-level retry
    Retry,
    /// Non-retryable error (deserialization, serialization), move to dead letter queue
    DeadLetterQueue(DeadLetterQueueReason),
}

/// Worker that processes S3 uploads with bounded parallelism
pub struct Worker {
    /// Worker ID (for logging/debugging)
    id: usize,
    /// Shared work queue (MPMC via SegQueue)
    work_queue: Arc<SegQueue<WorkItem>>,
    /// Notification for work availability
    work_available: Arc<Notify>,
    /// Bounded set of concurrent upload tasks
    joinset: JoinSet<UploadResult>,
    /// Maximum concurrent uploads
    max_parallel: usize,
    /// Current number of active uploads (shared with coordinator)
    active_uploads: Arc<AtomicUsize>,
    /// Disk store for reading/removing events
    disk_store: Arc<S3DiskStore>,
    /// S3 client (cloned from coordinator)
    s3_client: Client,
    /// S3 bucket name
    bucket_name: String,
    /// Cache name (for metrics/logging)
    cache_name: String,
    /// Shutdown signal
    shutdown: Arc<Notify>,
}

impl Worker {
    /// Create a new worker
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: usize,
        work_queue: Arc<SegQueue<WorkItem>>,
        work_available: Arc<Notify>,
        max_parallel: usize,
        disk_store: Arc<S3DiskStore>,
        s3_client: Client,
        bucket_name: String,
        cache_name: String,
        shutdown: Arc<Notify>,
        active_uploads: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            id,
            work_queue,
            work_available,
            joinset: JoinSet::new(),
            max_parallel,
            active_uploads,
            disk_store,
            s3_client,
            bucket_name,
            cache_name,
            shutdown,
        }
    }

    /// Run the worker loop
    pub async fn run(mut self) {
        debug!(worker_id = self.id, cache_name = %self.cache_name, "Worker starting");

        let mut shutdown_signaled = false;

        loop {
            // Wait for work available OR task completion OR shutdown
            tokio::select! {
                // Just wake up when work available
                _ = self.work_available.notified() => {}

                // Handle completed upload
                Some(result) = self.joinset.join_next() => {
                    // Update active uploads count after task completes
                    self.active_uploads.store(self.joinset.len(), Ordering::Relaxed);

                    match result {
                        Ok(upload_result) => {
                            if matches!(upload_result.outcome, UploadOutcome::Retry) {
                                self.work_queue.push(upload_result.work_item);
                            }
                        }
                        Err(join_err) => {
                            error!(
                                worker_id = self.id,
                                cache_name = %self.cache_name,
                                error = ?join_err,
                                "Worker task panicked"
                            );
                        }
                    }
                }

                // Shutdown signal (one-shot, track with flag)
                _ = self.shutdown.notified() => {
                    debug!(
                        worker_id = self.id,
                        cache_name = %self.cache_name,
                        "Shutdown signal received"
                    );
                    shutdown_signaled = true;
                }
            }

            // Exit only when shutdown signaled AND both queues empty
            if shutdown_signaled && self.joinset.is_empty() && self.work_queue.is_empty() {
                debug!(
                    worker_id = self.id,
                    cache_name = %self.cache_name,
                    "Worker shutting down - queues drained"
                );
                break;
            }

            // After ANY wakeup, drain queue while we have capacity
            while self.joinset.len() < self.max_parallel {
                if let Some(work_item) = self.work_queue.pop() {
                    self.spawn_upload(work_item);
                } else {
                    break;
                }
            }
        }

        debug!(worker_id = self.id, cache_name = %self.cache_name, "Worker stopped");
    }

    /// Spawn an upload task in the JoinSet
    fn spawn_upload(&mut self, work_item: WorkItem) {
        let disk_store = Arc::clone(&self.disk_store);
        let s3_client = self.s3_client.clone();
        let bucket_name = self.bucket_name.clone();
        let cache_name = self.cache_name.clone();

        self.joinset.spawn(async move {
            Self::upload_task(work_item, disk_store, s3_client, bucket_name, cache_name).await
        });

        // Update active uploads count after spawning
        self.active_uploads
            .store(self.joinset.len(), Ordering::Relaxed);
    }

    /// Upload task: read from disk, upload to S3, return result
    async fn upload_task(
        work_item: WorkItem,
        disk_store: Arc<S3DiskStore>,
        s3_client: Client,
        bucket_name: String,
        cache_name: String,
    ) -> UploadResult {
        use std::time::Instant;

        // Read event from disk (deserialization happens here)
        let event = match disk_store.read_event(work_item.ulid.into()) {
            Ok(event) => event,
            Err(e) => {
                error!(
                    error = ?e,
                    ulid = %work_item.ulid,
                    cache_name = %cache_name,
                    "Failed to read event from disk"
                );
                if let Err(dlq_error) = disk_store.move_to_dead_letter_queue(work_item.ulid.into())
                {
                    error!(
                        error = ?dlq_error,
                        ulid = %work_item.ulid,
                        cache_name = %cache_name,
                        "Failed to move event to dead letter queue"
                    );
                }
                monotonic!(
                    s3_write_attempts = 1,
                    cache_name = &cache_name,
                    backend = "s3",
                    result = "dead_letter_queue"
                );
                return UploadResult {
                    work_item,
                    outcome: UploadOutcome::DeadLetterQueue(DeadLetterQueueReason::DiskReadError(
                        e,
                    )),
                };
            }
        };

        // Upload to S3 (SDK handles retries)
        let start = Instant::now();
        let key = event.key.as_ref();
        let body = event.payload.value.as_ref();

        let result = s3_client
            .put_object()
            .bucket(&bucket_name)
            .key(key)
            .body(body.to_vec().into())
            .send()
            .await;

        let duration_ms = start.elapsed().as_millis() as f64;
        let event_kind = event.event_kind.as_ref();

        match result {
            Ok(_) => {
                // Record success metrics
                histogram!(
                    layer_cache_persister.write_duration_ms = duration_ms,
                    cache_name = &cache_name,
                    backend = "s3",
                    status = "success",
                    event_kind = &event_kind
                );

                histogram!(
                    layer_cache_persistence_latency_seconds = chrono::Utc::now()
                        .signed_duration_since(event.metadata.timestamp)
                        .to_std()
                        .unwrap_or_default()
                        .as_secs_f64(),
                    cache_name = &cache_name,
                    backend = "s3",
                    operation = "write",
                    event_kind = &event_kind
                );

                if let Err(e) = disk_store.remove(work_item.ulid.into()) {
                    error!(
                        error = ?e,
                        ulid = %work_item.ulid,
                        cache_name = %cache_name,
                        "Failed to remove successfully uploaded event from disk"
                    );
                }

                monotonic!(
                    s3_write_attempts = 1,
                    cache_name = &cache_name,
                    backend = "s3",
                    result = "success"
                );

                UploadResult {
                    work_item,
                    outcome: UploadOutcome::Success,
                }
            }
            Err(_) => {
                // Record error metrics
                histogram!(
                    layer_cache_persister.write_duration_ms = duration_ms,
                    cache_name = &cache_name,
                    backend = "s3",
                    status = "error",
                    event_kind = &event_kind
                );

                monotonic!(
                    s3_write_attempts = 1,
                    cache_name = &cache_name,
                    backend = "s3",
                    result = "retry"
                );

                UploadResult {
                    work_item,
                    outcome: UploadOutcome::Retry,
                }
            }
        }
    }
}
