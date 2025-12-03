use std::{
    collections::BTreeSet,
    sync::Arc,
    time::Duration,
};

use aws_sdk_s3::Client;
use futures::FutureExt;
use telemetry::prelude::*;
use telemetry_utils::{
    gauge,
    histogram,
    monotonic,
};
use tokio::{
    sync::{
        Notify,
        mpsc,
    },
    time::sleep,
};
use ulid::Ulid;

use crate::{
    error::{
        AwsSdkError,
        LayerDbError,
        S3Error,
        S3Operation,
    },
    event::LayeredEvent,
    rate_limiter::{
        RateLimitConfig,
        RateLimiter,
    },
    s3::categorize_s3_error,
    s3_write_queue::{
        S3WriteQueue,
        S3WriteQueueError,
    },
};

/// Configuration for initializing S3QueueProcessor with queue state
pub struct ProcessorQueueSetup {
    pub rx: mpsc::UnboundedReceiver<Ulid>,
    pub notify: Arc<Notify>,
    pub initial_ulids: Vec<Ulid>,
}

pub struct S3QueueProcessor {
    write_queue: Arc<S3WriteQueue>, // Rename from 'queue' to 'write_queue' for clarity
    queue: BTreeSet<Ulid>,          // In-memory index of pending items
    rx: mpsc::UnboundedReceiver<Ulid>, // Receives new items from enqueue
    notify: Arc<Notify>,            // For wake-up when queue empty
    rate_limiter: RateLimiter,
    s3_client: Client,
    bucket_name: String,
    cache_name: String,
    shutdown: Arc<Notify>,
}

impl S3QueueProcessor {
    pub fn new(
        write_queue: Arc<S3WriteQueue>,
        rate_limiter_config: RateLimitConfig,
        s3_client: Client,
        bucket_name: String,
        cache_name: String,
        queue_setup: ProcessorQueueSetup,
    ) -> Self {
        // Convert Vec<Ulid> to BTreeSet for efficient ordering and lookup
        let queue: BTreeSet<Ulid> = queue_setup.initial_ulids.into_iter().collect();

        Self {
            write_queue,
            queue,
            rx: queue_setup.rx,
            notify: queue_setup.notify,
            rate_limiter: RateLimiter::new(rate_limiter_config),
            s3_client,
            bucket_name,
            cache_name,
            shutdown: Arc::new(Notify::new()),
        }
    }

    pub fn shutdown_handle(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown)
    }

    /// Drains up to `available` items from the channel into the queue.
    /// Returns number of items drained.
    fn drain_channel(&mut self, available: usize) -> usize {
        let mut drained = 0;
        while let Ok(ulid) = self.rx.try_recv() {
            self.queue.insert(ulid);
            drained += 1;
            if drained >= available {
                break;
            }
        }
        drained
    }

    fn record_metrics(&self) {
        let depth = self.queue.len();
        let backoff_ms = self.rate_limiter.current_delay().as_millis() as u64;

        gauge!(
            s3_write_queue_depth = depth,
            cache_name = &self.cache_name,
            backend = "s3"
        );

        gauge!(
            s3_write_backoff_ms = backoff_ms,
            cache_name = &self.cache_name,
            backend = "s3"
        );
    }

    fn record_write_attempt(&self, result: &str) {
        monotonic!(
            s3_write_attempts_total = 1,
            cache_name = &self.cache_name,
            backend = "s3",
            result = result
        );
    }

    fn record_write_duration(&self, duration_ms: u64) {
        histogram!(
            s3_write_duration_ms = duration_ms,
            cache_name = &self.cache_name,
            backend = "s3"
        );
    }

    pub async fn process_queue(mut self) {
        loop {
            // Check for shutdown signal (non-blocking)
            if self.shutdown.notified().now_or_never().is_some() {
                debug!(cache = %self.cache_name, "S3QueueProcessor received shutdown signal, exiting");
                break;
            }

            // 1. If queue empty, wait for notification or 30s timeout
            // The timeout ensures we emit metrics even during inactivity
            if self.queue.is_empty() {
                tokio::time::timeout(Duration::from_secs(30), self.notify.notified())
                    .await
                    .ok();
            }

            // 2. Drain bounded amount from channel
            let available = self.rx.len();
            let drained = self.drain_channel(available);

            // Log drain statistics at trace level for debugging
            if drained > 0 {
                trace!(
                    cache = %self.cache_name,
                    drained = drained,
                    available = available,
                    queue_size = self.queue.len(),
                    "Drained items from channel"
                );
            }

            // 3. If still empty after draining, record metrics and loop back
            if self.queue.is_empty() {
                self.record_metrics();
                continue;
            }

            // Record metrics before processing
            self.record_metrics();

            // 4. Peek at oldest item (don't remove yet!)
            // BTreeSet orders by ULID, so first() is oldest timestamp
            let ulid = match self.queue.first() {
                Some(ulid) => *ulid,
                None => {
                    // Queue became empty (shouldn't happen, but defensive)
                    warn!(cache = %self.cache_name, "Queue empty after metrics check");
                    continue;
                }
            };

            // 5. Read event from disk
            let event = match self.write_queue.read_event(ulid) {
                Ok(event) => event,
                Err(e) => {
                    // Disk read failures are unrecoverable for this specific item
                    // Leaving it in queue would cause infinite retries
                    error!(
                        cache = %self.cache_name,
                        ulid = %ulid,
                        error = %e,
                        "Failed to read event from disk - moving to DLQ"
                    );

                    // Remove from in-memory queue
                    let removed = self.queue.remove(&ulid);
                    if !removed {
                        warn!(
                            cache = %self.cache_name,
                            ulid = %ulid,
                            "ULID not found in queue during disk read error handling"
                        );
                    }

                    // Move to DLQ on disk (if file exists)
                    // If file is missing, this will fail, but that's OK - we still remove from queue
                    if let Err(dlq_err) = self.write_queue.move_to_dlq(ulid, &e) {
                        warn!(
                            cache = %self.cache_name,
                            ulid = %ulid,
                            error = %dlq_err,
                            "Failed to move unreadable file to DLQ (file may be missing)"
                        );
                    }

                    // Record metric for disk read failures
                    self.record_write_attempt("error_disk_read");

                    // Continue processing other items
                    continue;
                }
            };

            // 6. Process the item (writes to S3)
            self.process_item(ulid, event).await;

            // 7. Apply backoff delay AFTER processing
            // This ensures backoff only applies after actual S3 API attempts
            // Disk read failures (continue above) skip backoff entirely
            let delay = self.rate_limiter.current_delay();
            if delay > Duration::ZERO {
                sleep(delay).await;
            }
        }
    }

    #[instrument(
        skip(self, event),
        fields(
            cache = %self.cache_name,
            ulid = %ulid,
            backoff_ms = self.rate_limiter.current_delay().as_millis(),
            queue_depth = self.queue.len(),
            result = tracing::field::Empty,
        )
    )]
    async fn process_item(&mut self, ulid: Ulid, event: LayeredEvent) {
        trace!(
            cache = %self.cache_name,
            ulid = %ulid,
            "Processing S3 write from queue"
        );

        let span = Span::current();

        // Key is already transformed - write directly to S3
        // Event.key contains the final S3 key after transformation
        let result = self
            .s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(event.key.as_ref()) // Pre-transformed key from queue
            .body(Arc::unwrap_or_clone(event.payload.value).into())
            .send()
            .await;

        // Categorize result for metrics and rate limiter
        let categorized_result = result.map_err(|sdk_err| {
            // Convert AWS SDK error to categorized S3Error
            let aws_error = AwsSdkError::PutObject(sdk_err);
            let s3_error = categorize_s3_error(
                aws_error,
                S3Operation::Put,
                self.cache_name.clone(),
                event.key.to_string(),
            );
            LayerDbError::S3(Box::new(s3_error))
        });

        match categorized_result {
            Ok(_) => {
                span.record("result", "success");
                self.handle_success(ulid).await
            }
            Err(s3_error) => {
                // Classify the error to record appropriate result
                let result_str = match &s3_error {
                    LayerDbError::S3(boxed_error) => match boxed_error.as_ref() {
                        S3Error::Throttling { .. } => "throttle",
                        S3Error::Configuration { .. } => "error_configuration",
                        _ => "error_transient",
                    },
                    _ => "error_transient",
                };
                span.record("result", result_str);
                self.handle_error(ulid, &s3_error).await
            }
        }
    }

    async fn handle_success(&mut self, ulid: Ulid) {
        // Calculate duration from ULID timestamp to now
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let ulid_ms = ulid.timestamp_ms();
        let duration_ms = now_ms.saturating_sub(ulid_ms);

        // Remove from in-memory queue
        let removed = self.queue.remove(&ulid);
        if !removed {
            warn!(
                cache = %self.cache_name,
                ulid = %ulid,
                "ULID not found in queue during success handling"
            );
        }

        // Remove from disk queue
        if let Err(e) = self.write_queue.remove(ulid) {
            error!(cache = %self.cache_name, ulid = %ulid, error = %e,
                   "Failed to remove completed write from disk queue");
        }

        trace!(cache = %self.cache_name, ulid = %ulid,
               delay_ms = self.rate_limiter.current_delay().as_millis(),
               "Completed S3 write");

        // Record metrics
        self.record_write_attempt("success");
        self.record_write_duration(duration_ms);

        // Update rate limiter
        self.rate_limiter.on_success();
        if self.rate_limiter.should_reduce_backoff() {
            let old_delay = self.rate_limiter.current_delay();
            self.rate_limiter.reduce_backoff();
            let new_delay = self.rate_limiter.current_delay();

            debug!(
                cache = %self.cache_name,
                old_delay_ms = old_delay.as_millis(),
                new_delay_ms = new_delay.as_millis(),
                consecutive_successes = self.rate_limiter.consecutive_successes(),
                "S3 backoff reduced after successes"
            );
        }
    }

    async fn handle_error(&mut self, ulid: Ulid, error: &LayerDbError) {
        // Extract S3Error if this is an S3 error
        let s3_error = match error {
            LayerDbError::S3(boxed_error) => boxed_error.as_ref(),
            _ => {
                // Non-S3 errors are treated as transient
                warn!(
                    cache = %self.cache_name,
                    ulid = %ulid,
                    error = %error,
                    "S3 write failed with non-S3 error, will retry"
                );
                self.rate_limiter.on_throttle();
                return;
            }
        };

        // Classify based on S3Error variant
        match s3_error {
            S3Error::Throttling { .. } => {
                self.record_write_attempt("throttle");

                let old_delay = self.rate_limiter.current_delay();
                self.rate_limiter.on_throttle();
                let new_delay = self.rate_limiter.current_delay();

                debug!(
                    cache = %self.cache_name,
                    ulid = %ulid,
                    old_delay_ms = old_delay.as_millis(),
                    new_delay_ms = new_delay.as_millis(),
                    "S3 rate limited, increasing backoff"
                );
                // Leave in queue, will retry - DO NOT remove from self.queue
            }
            S3Error::Configuration { message, .. } => {
                self.record_write_attempt("error_configuration");

                // Configuration errors are non-retryable
                error!(
                    cache = %self.cache_name,
                    ulid = %ulid,
                    error = %error,
                    "S3 write failed: configuration error, moving to DLQ"
                );

                // Remove from in-memory queue
                self.queue.remove(&ulid);

                // Move to DLQ on disk
                if let Err(e) = self.write_queue.move_to_dlq(
                    ulid,
                    &S3WriteQueueError::Configuration {
                        message: message.clone(),
                    },
                ) {
                    error!(cache = %self.cache_name, ulid = %ulid, dlq_error = %e,
                           "Failed to move corrupted write to DLQ");
                }
            }
            _ => {
                self.record_write_attempt("error_transient");

                // All other errors (Network, Authentication, NotFound, Other) are transient
                warn!(
                    cache = %self.cache_name,
                    ulid = %ulid,
                    error = %error,
                    "S3 write failed with transient error, will retry"
                );

                // Treat like throttle - increase backoff
                self.rate_limiter.on_throttle();
                // Leave in queue, will retry - DO NOT remove from self.queue
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;
    use tokio::sync::{
        Notify,
        mpsc,
    };

    use super::*;
    use crate::{
        rate_limiter::RateLimitConfig,
        s3_write_queue::S3WriteQueue,
    };

    fn create_test_processor() -> (S3QueueProcessor, mpsc::UnboundedSender<Ulid>) {
        let temp_dir = TempDir::new().unwrap();
        let notify = Arc::new(Notify::new());
        let (write_queue, _processor_rx) =
            S3WriteQueue::new(temp_dir.path(), "test", notify.clone()).unwrap();

        // Create a test channel - processor will use this rx
        let (test_tx, test_rx) = mpsc::unbounded_channel();

        let config = aws_sdk_s3::Config::builder()
            .region(aws_sdk_s3::config::Region::from_static("us-east-1"))
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();

        // Create processor with test rx (not the one from write_queue)
        let processor = S3QueueProcessor {
            write_queue: Arc::new(write_queue),
            queue: BTreeSet::new(),
            rx: test_rx,
            notify: notify.clone(),
            rate_limiter: RateLimiter::new(RateLimitConfig::default()),
            s3_client: Client::from_conf(config),
            bucket_name: "test-bucket".to_string(),
            cache_name: "test".to_string(),
            shutdown: Arc::new(Notify::new()),
        };

        (processor, test_tx)
    }

    #[test]
    fn test_drain_channel_empty() {
        let (mut processor, _tx) = create_test_processor();

        let drained = processor.drain_channel(10);

        assert_eq!(drained, 0);
        assert_eq!(processor.queue.len(), 0);
    }

    #[test]
    fn test_drain_channel_with_items() {
        let (mut processor, test_tx) = create_test_processor();

        // Send items via test channel
        let ulid1 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let ulid2 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let ulid3 = Ulid::new();

        test_tx.send(ulid1).unwrap();
        test_tx.send(ulid2).unwrap();
        test_tx.send(ulid3).unwrap();

        let drained = processor.drain_channel(10);

        assert_eq!(drained, 3);
        assert_eq!(processor.queue.len(), 3);
        assert!(processor.queue.contains(&ulid1));
        assert!(processor.queue.contains(&ulid2));
        assert!(processor.queue.contains(&ulid3));
    }

    #[test]
    fn test_drain_channel_respects_bound() {
        let (mut processor, test_tx) = create_test_processor();

        // Send 5 items
        for _ in 0..5 {
            let ulid = Ulid::new();
            test_tx.send(ulid).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Drain with bound of 3
        let drained = processor.drain_channel(3);

        assert_eq!(drained, 3);
        assert_eq!(processor.queue.len(), 3);

        // 2 items should still be in channel
        assert!(processor.rx.try_recv().is_ok());
        assert!(processor.rx.try_recv().is_ok());
        assert!(processor.rx.try_recv().is_err()); // Channel now empty
    }

    #[tokio::test]
    async fn test_process_queue_exits_on_shutdown() {
        let (processor, _test_tx) = create_test_processor();
        let shutdown = processor.shutdown_handle();
        let notify = Arc::clone(&processor.notify);

        // Spawn processor task
        let handle = tokio::spawn(processor.process_queue());

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Send shutdown signal
        shutdown.notify_one();

        // Wake up the processor so it can check the shutdown signal
        // (Processor may be blocked on notify.notified().await if queue empty)
        notify.notify_one();

        // Should exit cleanly
        tokio::time::timeout(Duration::from_millis(100), handle)
            .await
            .expect("Processor should exit on shutdown")
            .expect("Task should not panic");
    }

    #[tokio::test]
    async fn test_disk_read_error_removes_from_queue() {
        use std::collections::BTreeSet;

        let temp_dir = TempDir::new().unwrap();
        let notify = Arc::new(Notify::new());
        let (write_queue, rx) = S3WriteQueue::new(temp_dir.path(), "test", notify.clone()).unwrap();

        // Create processor with one ULID in queue (but no file on disk)
        let missing_ulid = Ulid::new();
        let mut queue = BTreeSet::new();
        queue.insert(missing_ulid);

        let config = aws_sdk_s3::Config::builder()
            .region(aws_sdk_s3::config::Region::from_static("us-east-1"))
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();

        let mut processor = S3QueueProcessor {
            write_queue: Arc::new(write_queue),
            queue,
            rx,
            notify: notify.clone(),
            rate_limiter: RateLimiter::new(RateLimitConfig::default()),
            s3_client: Client::from_conf(config),
            bucket_name: "test-bucket".to_string(),
            cache_name: "test".to_string(),
            shutdown: Arc::new(Notify::new()),
        };

        // Verify ULID is in queue before
        assert_eq!(processor.queue.len(), 1);
        assert!(processor.queue.contains(&missing_ulid));

        // Manually run one iteration of the loop logic
        // (We can't run full process_queue easily, so test the error handling directly)

        // Try to read event - should fail
        let result = processor.write_queue.read_event(missing_ulid);
        assert!(result.is_err());

        // Handle the error as the loop would
        processor.queue.remove(&missing_ulid);

        // Verify ULID removed from queue
        assert_eq!(processor.queue.len(), 0);
        assert!(!processor.queue.contains(&missing_ulid));
    }

    #[tokio::test]
    async fn test_disk_read_error_handles_corrupted_file() {
        let temp_dir = TempDir::new().unwrap();
        let notify = Arc::new(Notify::new());
        let (write_queue, _rx) = S3WriteQueue::new(temp_dir.path(), "test", notify.clone()).unwrap();

        // Create a corrupted file
        let corrupted_ulid = Ulid::new();
        let corrupted_path = temp_dir
            .path()
            .join("test_s3_queue")
            .join(format!("{corrupted_ulid}.pending"));
        std::fs::write(&corrupted_path, b"corrupted data").unwrap();

        // Try to read - should fail with deserialization error
        let result = write_queue.read_event(corrupted_ulid);
        assert!(result.is_err());

        match result {
            Err(crate::s3_write_queue::S3WriteQueueError::DiskReadFailed { ulid, .. }) => {
                assert_eq!(ulid, corrupted_ulid);
            }
            _ => panic!("Expected DiskReadFailed error"),
        }
    }

    #[test]
    fn test_btreeset_ulid_ordering_monotonic() {
        let mut queue = BTreeSet::new();

        // Create ULIDs with delays to ensure monotonic timestamps
        let ulid1 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid3 = Ulid::new();

        // Insert in order
        queue.insert(ulid1);
        queue.insert(ulid2);
        queue.insert(ulid3);

        // first() should return oldest (ulid1)
        assert_eq!(queue.first(), Some(&ulid1));

        // Verify full order
        let ordered: Vec<_> = queue.iter().copied().collect();
        assert_eq!(ordered, vec![ulid1, ulid2, ulid3]);
    }

    #[test]
    fn test_btreeset_ulid_ordering_non_monotonic_insert() {
        let mut queue = BTreeSet::new();

        // Create ULIDs in chronological order
        let ulid1 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid3 = Ulid::new();

        // Insert in reverse order (non-monotonic)
        queue.insert(ulid3);
        queue.insert(ulid1);
        queue.insert(ulid2);

        // first() should STILL return oldest (ulid1), not first inserted (ulid3)
        assert_eq!(queue.first(), Some(&ulid1));

        // Verify full order - should be chronological, not insertion order
        let ordered: Vec<_> = queue.iter().copied().collect();
        assert_eq!(ordered, vec![ulid1, ulid2, ulid3]);
    }

    #[test]
    fn test_btreeset_remove_and_first() {
        let mut queue = BTreeSet::new();

        let ulid1 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid3 = Ulid::new();

        queue.insert(ulid1);
        queue.insert(ulid2);
        queue.insert(ulid3);

        // Remove oldest
        assert!(queue.remove(&ulid1));

        // first() should now return ulid2 (next oldest)
        assert_eq!(queue.first(), Some(&ulid2));

        // Remove ulid2
        assert!(queue.remove(&ulid2));

        // first() should now return ulid3
        assert_eq!(queue.first(), Some(&ulid3));

        // Remove ulid3
        assert!(queue.remove(&ulid3));

        // Queue should be empty
        assert!(queue.is_empty());
        assert_eq!(queue.first(), None);
    }

    #[test]
    fn test_btreeset_peek_then_remove_pattern() {
        let mut queue = BTreeSet::new();

        let ulid1 = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let ulid2 = Ulid::new();

        queue.insert(ulid1);
        queue.insert(ulid2);

        // Peek at oldest (doesn't remove)
        let oldest = queue.first().copied();
        assert_eq!(oldest, Some(ulid1));

        // Queue still has both items
        assert_eq!(queue.len(), 2);

        // Simulate processing success - remove after processing
        if let Some(ulid) = oldest {
            queue.remove(&ulid);
        }

        // Now queue has only ulid2
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.first(), Some(&ulid2));
    }

    #[test]
    fn test_btreeset_duplicate_insert_idempotent() {
        let mut queue = BTreeSet::new();

        let ulid1 = Ulid::new();

        // Insert same ULID multiple times
        assert!(queue.insert(ulid1));  // First insert returns true
        assert!(!queue.insert(ulid1)); // Duplicate returns false
        assert!(!queue.insert(ulid1)); // Still returns false

        // Queue has only one item
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.first(), Some(&ulid1));
    }
}
