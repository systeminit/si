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
    sync::mpsc::{
        self,
        error::{TryRecvError, TrySendError},
        Receiver, Sender,
    },
};
use tokio_util::sync::CancellationToken;

const DEFAULT_TICK_DURATION: Duration = Duration::from_millis(100);
const DEFAULT_WARN_THRESHOLD: Duration = Duration::from_millis(100);

pub fn new(
    runtime_name: impl Into<String>,
    handle: Handle,
    cancellation_token: CancellationToken,
) -> TokioWatchdog {
    TokioWatchdog::new(runtime_name.into(), handle, cancellation_token)
}

pub fn new_for_current(
    runtime_name: impl Into<String>,
    cancellation_token: CancellationToken,
) -> TokioWatchdog {
    TokioWatchdog::new(runtime_name.into(), Handle::current(), cancellation_token)
}

#[derive(Debug)]
pub struct TokioWatchdog {
    runtime_name: Box<str>,
    handle: Handle,
    tick_duration: Duration,
    warn_threshold: Duration,
    cancellation_token: CancellationToken,
}

impl TokioWatchdog {
    pub fn new(
        runtime_name: String,
        handle: Handle,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            runtime_name: runtime_name.into_boxed_str(),
            handle,
            tick_duration: DEFAULT_TICK_DURATION,
            warn_threshold: DEFAULT_WARN_THRESHOLD,
            cancellation_token,
        }
    }

    pub fn with_tick_duration(self, tick_duration: Duration) -> Self {
        Self {
            tick_duration,
            ..self
        }
    }

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
    ///
    /// # Metrics
    ///
    /// - `count.tokio_watchdog.sent`: Number of requests sent to the tokio watchdog.
    /// - `histogram.tokio_watchdog.delay`: The delay between sending a request and the tokio
    ///    watchdog receiving it.
    /// - `histogram.tokio_watchdog.response_delay`: The delay between the tokio watchdog receiving
    ///    a request and responding.
    /// - `count.tokio_watchdog.send_failures`: Number of times the tokio watchdog failed to send a
    ///    request.
    /// - `count.tokio_watchdog.respond_failures`: Number of times the tokio watchdog failed to
    ///    respond to a request.
    pub fn spawn(self) -> io::Result<thread::JoinHandle<()>> {
        let Self {
            runtime_name,
            handle,
            tick_duration,
            warn_threshold,
            cancellation_token,
        } = self;

        if (tick_duration + warn_threshold).is_zero() {
            warn!(
                runtime = runtime_name,
                "sum of tick and warn duration must be non-zero"
            );
            return Err(io::Error::other("invalid watchdog configuration"));
        }
        // Set up an async task in tokio that will receive a request and respond as quickly as
        // possible.
        let (tx_request, mut rx_request) = mpsc::channel::<Instant>(1);
        let (tx_response, rx_response) = mpsc::channel::<Duration>(1);

        handle.spawn(async move {
            loop {
                let start = match rx_request.recv().await {
                    Some(start) => start,
                    None => {
                        debug!(
                            "tokio watchdog responder task shutting down because watchdog tx closed"
                        );
                        return;
                    }
                };

                // The `start.elapsed()` time should be almost immediate and represents the time
                // from the watchdog thread channel send to when the runtime wakes this task to
                // read the value and immediately send it back on a response channel. A slow and/or
                // hanging runtime may take some time to wake this task to read the value. This is
                // what we are measuring.
                match tx_response.try_send(start.elapsed()) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {
                        metric!(counter.tokio_watchdog.send_failures = 1);
                        continue;
                    }
                    Err(TrySendError::Closed(_)) => {
                        debug!(
                            "tokio watchdog responder task shutting down because watchdog rx closed"
                        );
                        return;
                    }
                }
            }
        });

        thread::Builder::new()
            .name(format!("tokio watchdog for {runtime_name}"))
            .spawn(move || {
                run_tokio_watchdog(
                    runtime_name,
                    tick_duration,
                    warn_threshold,
                    tx_request,
                    rx_response,
                    cancellation_token,
                )
            })
    }
}

// This blocks forever (at least until cancellation happens)
fn run_tokio_watchdog(
    runtime_name: Box<str>,
    tick_duration: Duration,
    warn_threshold: Duration,
    tx_request: Sender<Instant>,
    mut rx_response: Receiver<Duration>,
    cancellation_token: CancellationToken,
) {
    while !cancellation_token.is_cancelled() {
        // Sleep thread until next tick
        thread::sleep(tick_duration);

        // Send the request, including the time the request was sent so the receiver
        // can figure out the delay.
        match tx_request.try_send(Instant::now()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                metric!(counter.tokio_watchdog.send_failures = 1);
                continue;
            }
            Err(TrySendError::Closed(_)) => {
                error!("tokio watchdog shutting down because responder closed; shutting down");
                return;
            }
        }

        // Note that we sent the request
        metric!(count.tokio_watchdog.sent = 1);

        // Sleep thread until for `warn_threshold` time.
        //
        // This blocks the thread; we don't use async here because that would put us on the tokio
        // reactor.
        thread::sleep(warn_threshold);

        // Immediately try to read the `delay` value from the channel.
        let delay = match rx_response.try_recv() {
            // A value is immediately available within the `warn_threshold` time period.
            //
            // This signals a responsive and healthy runtime.
            Ok(delay) => delay,
            // There is not yet a value ready on the response channel.
            //
            // This means it will have taken longer than `warn_threshold` time to receive a
            // response. By our definition, the runtime is slow and/or hangining and therefore is
            // not healthy.
            Err(TryRecvError::Empty) => {
                // Log and count that the runtime is unhealty
                warn!(runtime = runtime_name, "tokio runtime starts hanging",);
                metric!(counter.tokio_watchdog.hang = 1);

                // Now we block the watchdog thread and wait for the response on the channel. At
                // this point we know the runtime is not healthy.
                let delay = match rx_response.blocking_recv() {
                    Some(delay) => delay,
                    // The sender has closed the channel.
                    None => {
                        debug!("tokio watchdog responder task tx has closed; shutting down");
                        return;
                    }
                };

                warn!(
                    runtime = runtime_name,
                    hang_nanos = delay.as_nanos(),
                    "tokio runtime has stops hanging",
                );

                delay
            }
            // The sender has closed the channel.
            Err(TryRecvError::Disconnected) => {
                debug!("tokio watchdog responder task tx has closed; shutting down");
                return;
            }
        };

        metric!(histogram.tokio_watchdog.response_time_nanos = delay.as_nanos());
    }

    debug!("tokio watchdog for {runtime_name} cancelled; shutting down");
}
