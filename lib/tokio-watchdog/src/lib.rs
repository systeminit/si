//! This crate supports metrics for how loaded the tokio reactor is.
#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::{
    io, thread,
    time::{Duration, Instant},
};

use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio::{
    runtime::Handle,
    sync::mpsc::{self, error::TrySendError, Receiver, Sender},
};
use tokio_util::sync::CancellationToken;

/// How frequently to emit metrics about the tokio reactor.
const DEFAULT_MEASUREMENT_FREQUENCY: Duration = Duration::from_secs(30);
/// The time before we record a warning that the watchdog hasn't received a response.
const DEFAULT_WARN_THRESHOLD: Duration = Duration::from_millis(200);
/// The maximum delay between being cancelled and the watchdog thread actually stopping.
/// The smaller this value, the more the watchdog will "spin" the thread, which could take
/// work away from other threads.
const MAX_CANCELLATION_DELAY: Duration = Duration::from_millis(200);

/// Spawn a new TokioWatchdog for the main tokio runtime.
///
/// # Metrics
///
/// - `count.tokio_watchdog.hang`: Whether or not a hang is currently happening.
/// - `histogram.tokio_watchdog.request_time_nanos`: The tokio reactor lag (time it took
///   for the watchdog receiver to wake when a new request was sent)
/// - `histogram.tokio_watchdog.global_queue_depth`: The depth of the tokio global queue
///   (should not generally have anything in it unless the system is heavily loaded)
/// - `histogram.tokio_watchdog.num_workers`: The number of workers in the tokio runtime
/// - `histogram.tokio_watchdog.num_alive_tasks`: The number of tokio tasks that are currently alive
pub fn spawn(
    bin_name: impl Into<Box<str>>,
    cancellation_token: CancellationToken,
) -> io::Result<thread::JoinHandle<()>> {
    TokioWatchdog::new(bin_name, cancellation_token).spawn()
}

/// Spawn a new TokioWatchdog for the current tokio runtime.
///
/// # Metrics
///
/// - `count.tokio_watchdog.hang`: Whether or not a hang is currently happening.
/// - `histogram.tokio_watchdog.request_time_nanos`: The tokio reactor lag (time it took
///   for the watchdog receiver to wake when a new request was sent)
/// - `histogram.tokio_watchdog.global_queue_depth`: The depth of the tokio global queue
///   (should not generally have anything in it unless the system is heavily loaded)
/// - `histogram.tokio_watchdog.num_workers`: The number of workers in the tokio runtime
/// - `histogram.tokio_watchdog.num_alive_tasks`: The number of tokio tasks that are currently alive
pub fn spawn_for_runtime(
    runtime_name: impl Into<Box<str>>,
    handle: Handle,
    cancellation_token: CancellationToken,
) -> io::Result<thread::JoinHandle<()>> {
    TokioWatchdog::new_for_runtime(runtime_name, handle, cancellation_token).spawn()
}

/// Watchdog for measuring the delay in the tokio reactor.
#[derive(Debug)]
pub struct TokioWatchdog {
    runtime_name: Box<str>,
    handle: Handle,
    measurement_frequency: Duration,
    warn_threshold: Duration,
    cancellation_token: CancellationToken,
}

/// Used as an error to signal that the watchdog was cancelled.
enum WatchdogError {
    Cancelled,
    InternalError,
}

impl TokioWatchdog {
    /// Create a new TokioWatchdog for the current Tokio runtime.
    pub fn new(runtime_name: impl Into<Box<str>>, cancellation_token: CancellationToken) -> Self {
        Self::new_for_runtime(runtime_name, Handle::current(), cancellation_token)
    }

    /// Create a new TokioWatchdog for a specific Tokio runtime.
    pub fn new_for_runtime(
        runtime_name: impl Into<Box<str>>,
        handle: Handle,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            runtime_name: runtime_name.into(),
            handle,
            measurement_frequency: DEFAULT_MEASUREMENT_FREQUENCY,
            warn_threshold: DEFAULT_WARN_THRESHOLD,
            cancellation_token,
        }
    }

    /// Set the frequency at which we measure the delay in the tokio reactor.
    pub fn with_measurement_frequency(self, measurement_frequency: Duration) -> Self {
        Self {
            measurement_frequency,
            ..self
        }
    }

    /// Set warn threshold
    pub fn with_warn_threshold(self, warn_threshold: Duration) -> Self {
        Self {
            warn_threshold,
            ..self
        }
    }

    /// Start a thread which instruments the delay in the tokio reactor.
    ///
    /// The thread periodically sends to a tokio channel and measures how long it takes before the
    /// tokio task receives the message.
    ///
    /// Returns a CancellationToken that can be used to cancel the watchdog.
    pub fn spawn(self) -> io::Result<thread::JoinHandle<()>> {
        if self.warn_threshold.is_zero() {
            warn!(
                runtime = self.runtime_name,
                "warn threshold must be above 0"
            );
            return Err(io::Error::other("invalid watchdog configuration"));
        }
        if self.measurement_frequency <= self.warn_threshold {
            warn!(
                runtime = self.runtime_name,
                "measurement frequency must be longer than warn threshold"
            );
            return Err(io::Error::other("invalid watchdog configuration"));
        }

        self.spawn_watchdog_loop()
    }

    fn spawn_watchdog_loop(self) -> io::Result<thread::JoinHandle<()>> {
        thread::Builder::new()
            .name(format!("tokio watchdog for {}", self.runtime_name))
            .spawn(move || {
                // Throw away the error that tells us whether it was cancelled
                let _ = self.watchdog_loop();
            })
    }

    // This blocks forever (at least until cancellation happens)
    fn watchdog_loop(self) -> Result<(), WatchdogError> {
        // Set up an async task in tokio that will receive a request and respond as quickly as
        // possible.
        let tx_request = self.spawn_responder_loop();

        debug!(
            runtime = self.runtime_name,
            measurement_frequency_secs = self.measurement_frequency.as_secs_f64(),
            warn_threshold_secs = self.warn_threshold.as_secs_f64(),
            "tokio watchdog started",
        );

        let mut sent_at = Instant::now();
        loop {
            self.send_tokio_metrics();

            // Sleep thread until it's time to measure again
            self.blocking_sleep(self.measurement_frequency.saturating_sub(sent_at.elapsed()))?;

            // Send the request, including the time the request was sent so the receiver
            // can figure out the delay.
            sent_at = Instant::now();
            match tx_request.try_send(sent_at) {
                Ok(()) => {}
                // It should be impossible for the channel to be full. The fact that we looped
                // back around means we waited for the receiver to pick up the last request.
                Err(TrySendError::Full(_)) => {
                    error!("tokio watchdog internal error: channel is full");
                    return Err(WatchdogError::InternalError);
                }
                Err(TrySendError::Closed(_)) => {
                    error!("tokio watchdog shutting down because responder closed early; shutting down");
                    return Err(WatchdogError::InternalError);
                }
            }

            // If the request isn't processed within the warn_threshold, we'll log a warning.
            self.blocking_sleep(self.warn_threshold)?;
            if Self::not_received_yet(&tx_request) {
                // It's taking forever. Mark it as a hang!
                warn!(
                    runtime = self.runtime_name,
                    elapsed = sent_at.elapsed().as_millis(),
                    capacity = tx_request.capacity(),
                    is_closed = tx_request.is_closed(),
                    "tokio watchdog hang detected"
                );
                metric!(counter.tokio_watchdog.hang = 1);

                // Wait for the responder to pick up the work (at which point the hang is over).
                // If we don't do this, the next iteration of the loop will have to handle
                // "channel is full" anyway, so may as well do it here.
                // TODO waiting until the responder picks up will delay us emitting tokio
                // metrics. We should cancel and maybe even restart the responder if we take
                // too long (or something like that).
                while Self::not_received_yet(&tx_request) {
                    self.blocking_sleep(MAX_CANCELLATION_DELAY)?;
                }

                // The responder finally picked up the work; we know we're not hanging now.
                info!(
                    runtime = self.runtime_name,
                    hang_nanos = sent_at.elapsed().as_nanos(),
                    "tokio watchdog hang ended"
                );
                metric!(counter.tokio_watchdog.hang = -1);
            }
        }
    }

    /// true if the request channel still has a value that hasn't been received, *and*
    /// the receiver is alive to receive it.
    fn not_received_yet(tx_request: &Sender<Instant>) -> bool {
        tx_request.capacity() == 0 && !tx_request.is_closed()
    }

    /// Blocks the thread for the given duration without invoking the tokio reactor.
    /// Will cancel the sleep early if the watchdog is cancelled.
    fn blocking_sleep(&self, mut duration: Duration) -> Result<(), WatchdogError> {
        // Sleep for small increments, wake up, and check for cancellation
        while duration > MAX_CANCELLATION_DELAY && self.check_cancelled()? {
            thread::sleep(MAX_CANCELLATION_DELAY);
            duration -= MAX_CANCELLATION_DELAY;
        }
        if self.check_cancelled()? && duration > Duration::ZERO {
            thread::sleep(duration);
        }
        Ok(())
    }

    // Check if we've been cancelled and toss an error if so.
    // Returns true if we haven't been cancelled, so it can be used in while loops.
    fn check_cancelled(&self) -> Result<bool, WatchdogError> {
        if self.cancellation_token.is_cancelled() {
            return Err(WatchdogError::Cancelled);
        }
        Ok(true)
    }

    fn spawn_responder_loop(&self) -> Sender<Instant> {
        let (tx_request, rx_request) = mpsc::channel::<Instant>(1);
        self.handle.spawn(Self::responder_loop(
            rx_request,
            self.runtime_name.to_string(),
        ));
        tx_request
    }

    /// Loop that receives requests on the tokio threads and records a metric of how long
    /// they took
    async fn responder_loop(mut rx_request: Receiver<Instant>, runtime_name: String) {
        loop {
            match rx_request.recv().await {
                Some(start) => {
                    // Record the metric
                    let request_time_nanos = start.elapsed().as_nanos() as u64;
                    metric!(
                        histogram.tokio_watchdog.request_time_nanos = request_time_nanos,
                        runtime = runtime_name
                    );
                }
                None => {
                    debug!(
                        "tokio watchdog responder task shutting down because watchdog tx closed"
                    );
                    return;
                }
            };
        }
    }

    fn send_tokio_metrics(&self) {
        let metrics = self.handle.metrics();
        metric!(
            histogram.tokio_watchdog.global_queue_depth = metrics.global_queue_depth(),
            runtime = self.runtime_name
        );
        metric!(
            histogram.tokio_watchdog.num_workers = metrics.num_workers(),
            runtime = self.runtime_name
        );
        metric!(
            histogram.tokio_watchdog.num_alive_tasks = metrics.num_alive_tasks(),
            runtime = self.runtime_name
        );
    }
}
