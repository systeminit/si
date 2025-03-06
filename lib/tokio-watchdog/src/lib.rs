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

use std::time::{Duration, Instant};
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio::sync::mpsc::{self, error::TrySendError};
use tokio_util::sync::CancellationToken;

/// Start a thread which instruments the delay in the tokio reactor.
///
/// The thread periodically sends to a tokio channel and measures how long it takes before
/// the tokio task receives the message.
///
/// Returns a CancellationToken that can be used to cancel the watchdog.
///
/// Reports metrics:
/// - count.tokio_watchdog.sent: Number of requests sent to the tokio watchdog.
/// - histogram.tokio_watchdog.delay: The delay between sending a request and the tokio watchdog receiving it.
/// - histogram.tokio_watchdog.response_delay: The delay between the tokio watchdog receiving a request and responding.
/// - count.tokio_watchdog.send_failures: Number of times the tokio watchdog failed to send a request.
/// - count.tokio_watchdog.respond_failures: Number of times the tokio watchdog failed to respond to a request.
pub fn spawn_tokio_watchdog(
    frequency: Duration,
    cancellation_token: CancellationToken,
) -> std::io::Result<CancellationToken> {
    let result = cancellation_token.clone();
    std::thread::Builder::new()
        .name(format!("tokio watchdog"))
        .spawn(move || run_tokio_watchdog(frequency, cancellation_token))?;
    Ok(result)
}

// This blocks forever (at least until cancellation happens)
fn run_tokio_watchdog(frequency: Duration, cancellation_token: CancellationToken) {
    // Set up an async task in tokio that will receive a request and respond as quickly as possible.
    let (tx_request, mut rx_request) = mpsc::channel::<Instant>(1);

    tokio::spawn(async move {
        loop {
            let delay = match rx_request.recv().await {
                Some(start) => start.elapsed(),
                None => {
                    debug!("tokio watchdog responder task shutting down because watchdog closed");
                    return;
                }
            };

            metric!(histogram.tokio_watchdog.delay = delay.as_nanos());
        }
    });

    // Give a moment for the spawn to start before we start measuring
    std::thread::sleep(frequency);

    while !cancellation_token.is_cancelled() {
        // Send the request, including the time the request was sent so the receiver
        // can figure out the delay.
        match tx_request.try_send(Instant::now()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                metric!(counter.tokio_watchdog.send_failures = 1);
                continue;
            }
            Err(TrySendError::Closed(_)) => {
                error!("tokio watchdog shutting down because responder closed");
                return;
            }
        }

        // Note that we sent the request
        metric!(count.tokio_watchdog.sent = 1);

        // Sleep until the next check. This blocks the thread; we don't use async here because
        // that would put us on the tokio reactor.
        std::thread::sleep(frequency);
    }

    debug!("tokio watchdog cancelled; shutting down");
    return;
}
