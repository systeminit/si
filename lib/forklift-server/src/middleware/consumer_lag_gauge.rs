//! Middleware for emitting consumer lag gauge metrics periodically.
//!
//! This middleware intercepts jetstream messages and periodically emits the `consumer_lag` gauge
//! metric based on the `pending` count from the message metadata. Uses a hybrid trigger approach:
//! emits every N seconds OR every M messages, whichever comes first.

use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{
            AtomicU64,
            Ordering,
        },
    },
    task::{
        Context,
        Poll,
    },
    time::Duration,
};

use naxum::Message;
use si_data_nats::async_nats::jetstream;
use tokio::{
    sync::Mutex,
    time::Instant,
};
use tower::{
    Layer,
    Service,
};

/// Default time threshold: emit lag metric every 10 seconds
const DEFAULT_TIME_THRESHOLD: Duration = Duration::from_secs(10);

/// Default message threshold: emit lag metric every 100 messages
const DEFAULT_MESSAGE_THRESHOLD: u64 = 100;

/// Layer for adding consumer lag gauge emission
#[derive(Clone)]
pub(crate) struct ConsumerLagGaugeLayer<F> {
    emit_fn: Arc<F>,
    time_threshold: Duration,
    message_threshold: u64,
}

impl<F> ConsumerLagGaugeLayer<F>
where
    F: Fn(u64) + Send + Sync + 'static,
{
    /// Create a new ConsumerLagGaugeLayer with the given emission function
    ///
    /// The function will be called with the lag value when the threshold is met.
    ///
    /// By default, the gauge will be emitted every 10 seconds OR every 100 messages,
    /// whichever comes first. Use `time_threshold()` and `message_threshold()` to customize.
    pub(crate) fn new(emit_fn: F) -> Self {
        Self {
            emit_fn: Arc::new(emit_fn),
            time_threshold: DEFAULT_TIME_THRESHOLD,
            message_threshold: DEFAULT_MESSAGE_THRESHOLD,
        }
    }

    /// Set the time threshold for emitting the lag gauge
    ///
    /// The gauge will be emitted when this duration has elapsed since the last emission,
    /// OR when the message threshold is reached, whichever comes first.
    #[allow(dead_code)]
    pub(crate) fn time_threshold(mut self, threshold: Duration) -> Self {
        self.time_threshold = threshold;
        self
    }

    /// Set the message count threshold for emitting the lag gauge
    ///
    /// The gauge will be emitted when this many messages have been processed since the
    /// last emission, OR when the time threshold is reached, whichever comes first.
    #[allow(dead_code)]
    pub(crate) fn message_threshold(mut self, threshold: u64) -> Self {
        self.message_threshold = threshold;
        self
    }
}

impl<S, F> Layer<S> for ConsumerLagGaugeLayer<F>
where
    F: Fn(u64) + Send + Sync + 'static,
{
    type Service = ConsumerLagGauge<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        ConsumerLagGauge {
            inner,
            emit_fn: self.emit_fn.clone(),
            time_threshold: self.time_threshold,
            message_threshold: self.message_threshold,
            last_report: Arc::new(Mutex::new(Instant::now())),
            messages_since_report: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Service that emits consumer lag gauge periodically
#[derive(Clone)]
pub(crate) struct ConsumerLagGauge<S, F> {
    inner: S,
    emit_fn: Arc<F>,
    time_threshold: Duration,
    message_threshold: u64,
    last_report: Arc<Mutex<Instant>>,
    messages_since_report: Arc<AtomicU64>,
}

impl<S, F> Service<Message<jetstream::Message>> for ConsumerLagGauge<S, F>
where
    S: Service<Message<jetstream::Message>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    F: Fn(u64) + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Message<jetstream::Message>) -> Self::Future {
        // Extract lag from message metadata before passing to inner service
        let lag = req.info().ok().map(|info| info.pending).unwrap_or(0);

        // Track messages and check if we should emit
        let messages_processed = self.messages_since_report.fetch_add(1, Ordering::Relaxed) + 1; // +1 because fetch_add returns previous value

        let last_report = self.last_report.clone();
        let messages_since_report = self.messages_since_report.clone();
        let emit_fn = self.emit_fn.clone();
        let time_threshold = self.time_threshold;
        let message_threshold = self.message_threshold;

        // Check thresholds and emit if needed
        let emit_future = async move {
            let mut should_emit = false;
            let mut last = last_report.lock().await;

            // Check time threshold
            if last.elapsed() >= time_threshold {
                should_emit = true;
            }

            // Check message count threshold
            if messages_processed >= message_threshold {
                should_emit = true;
            }

            if should_emit {
                // Call the emission function with the lag value
                (emit_fn)(lag);

                // Reset tracking
                *last = Instant::now();
                messages_since_report.store(0, Ordering::Relaxed);
            }
        };

        // Clone the service for the async block
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // Emit gauge if threshold met
            emit_future.await;

            // Pass request to inner service
            inner.call(req).await
        })
    }
}
