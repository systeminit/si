use std::{
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
    sync::Notify,
    time::sleep,
};
use ulid::Ulid;

use crate::{
    error::{
        LayerDbError,
        S3Error,
    },
    event::LayeredEvent,
    rate_limiter::{
        RateLimitConfig,
        RateLimiter,
    },
    s3_write_queue::{
        S3WriteQueue,
        S3WriteQueueError,
    },
};

pub struct S3QueueProcessor {
    queue: Arc<S3WriteQueue>,
    rate_limiter: RateLimiter,
    s3_client: Client,
    bucket_name: String,
    cache_name: String,
    shutdown: Arc<Notify>,
}

impl S3QueueProcessor {
    pub fn new(
        queue: Arc<S3WriteQueue>,
        rate_limiter_config: RateLimitConfig,
        s3_client: Client,
        bucket_name: String,
        cache_name: String,
    ) -> Self {
        Self {
            queue,
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

    fn record_metrics(&self) {
        let depth = self.queue.depth();
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

            // Record metrics before processing
            self.record_metrics();

            // Apply backoff delay if any
            let delay = self.rate_limiter.current_delay();
            if delay > Duration::ZERO {
                sleep(delay).await;
            }

            // Scan queue for next item
            let items = match self.queue.scan() {
                Ok(items) => items,
                Err(e) => {
                    error!(cache = %self.cache_name, "Failed to scan S3 write queue: {}", e);
                    sleep(Duration::from_secs(1)).await; // Brief pause before retry
                    continue;
                }
            };

            // Queue empty - wait briefly then check again
            if items.is_empty() {
                sleep(Duration::from_millis(100)).await;
                continue;
            }

            // Process oldest item (first in ULID order)
            let (ulid, event) = items.into_iter().next().unwrap();
            self.process_item(ulid, event).await;
        }
    }

    #[instrument(
        skip(self, event),
        fields(
            cache = %self.cache_name,
            ulid = %ulid,
            backoff_ms = self.rate_limiter.current_delay().as_millis(),
            queue_depth = self.queue.depth(),
            result = tracing::field::Empty,
        )
    )]
    async fn process_item(&mut self, ulid: Ulid, event: LayeredEvent) {
        trace!("Processing S3 write from queue");

        let span = Span::current();

        // Attempt S3 write using the key from the event
        let result = self.s3_layer.insert(&event.key, &event.payload.value).await;

        match result {
            Ok(_) => {
                span.record("result", "success");
                self.handle_success(ulid).await
            }
            Err(ref e) => {
                // Classify the error to record appropriate result
                let result_str = match e {
                    LayerDbError::S3(boxed_error) => match boxed_error.as_ref() {
                        S3Error::Throttling { .. } => "throttle",
                        S3Error::Configuration { .. } => "error_configuration",
                        _ => "error_transient",
                    },
                    _ => "error_transient",
                };
                span.record("result", result_str);
                self.handle_error(ulid, e).await
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

        // Remove from queue
        if let Err(e) = self.queue.remove(ulid) {
            error!(cache = %self.cache_name, ulid = %ulid, error = %e,
                   "Failed to remove completed write from queue");
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
                // Leave in queue, will retry
            }
            S3Error::Configuration { message, .. } => {
                self.record_write_attempt("error_configuration");

                // Configuration errors are non-retryable, similar to serialization errors
                error!(
                    cache = %self.cache_name,
                    ulid = %ulid,
                    error = %error,
                    "S3 write failed: configuration error, moving to DLQ"
                );

                if let Err(e) = self.queue.move_to_dlq(
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
                // Leave in queue, will retry
            }
        }
    }
}
