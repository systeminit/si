//! Graceful service/server shutdown using cancellation tokens, task trackers, driven by Unix
//! signal handling.

use std::{error, future::Future, io, time::Duration};

use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix::{self, SignalKind},
    time::timeout,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

/// An error that can be returned when gracefully shutting down.
///
/// See [`graceful`] for more details.
#[derive(Debug, Error)]
pub enum ShutdownError {
    /// When a signal handler fails to be correcly setup
    #[error("failed to setup signal handler: {0}")]
    Signal(#[source] io::Error),
    /// When there is a telemetry-related error
    #[error("telemetry shutdown error: {0}")]
    Telemetry(#[source] Box<dyn error::Error + Send + Sync + 'static>),
    /// When the timeout to wait for graceful shutdown has been exceeded
    #[error("graceful shutdown timeout elapsed: {0:?}")]
    TimeoutElapsed(Duration),
}

impl ShutdownError {
    fn telemetry<E>(err: E) -> Self
    where
        E: error::Error + Send + Sync + 'static,
    {
        Self::Telemetry(Box::new(err))
    }
}

/// Gracfully shutdown a service that may be running multiple tasks and in-flight work.
///
/// # Platform-specific behavior
///
/// This function sets up a signal handler for both `SIGINT` (i.e. `Ctrl+c`) and `SIGTERM` so usage
/// of this function with other code intercepting these signals is *highly* discouraged.
pub async fn graceful<Fut, E>(
    tracker: TaskTracker,
    token: CancellationToken,
    telemetry_guard: Option<Fut>,
    shutdown_timeout: Option<Duration>,
) -> Result<(), ShutdownError>
where
    Fut: Future<Output = Result<(), E>>,
    E: error::Error + Send + Sync + 'static,
{
    let mut sig_int = unix::signal(SignalKind::interrupt()).map_err(ShutdownError::Signal)?;
    let mut sig_term = unix::signal(SignalKind::terminate()).map_err(ShutdownError::Signal)?;

    tokio::select! {
        _ = sig_int.recv() => {
            info!("received SIGINT, performing graceful shutdown");
            tracker.close();
            token.cancel();
        }
        _ = sig_term.recv() => {
            info!("received SIGTERM, performing graceful shutdown");
            tracker.close();
            token.cancel();
        }
    }

    // Wait for all tasks to finish
    match shutdown_timeout {
        Some(duration) => {
            if let Err(_elapsed) = timeout(duration, tracker.wait()).await {
                warn!("graceful shutdown timeout exceeded; completing shutdown anyway");
                if let Some(telemetry_guard) = telemetry_guard {
                    // Wait for telemetry to shutdown
                    telemetry_guard.await.map_err(ShutdownError::telemetry)?;
                }
                return Err(ShutdownError::TimeoutElapsed(duration));
            }
        }
        None => {
            tracker.wait().await;
        }
    }

    if let Some(telemetry_guard) = telemetry_guard {
        // Wait for telemetry to shutdown
        telemetry_guard.await.map_err(ShutdownError::telemetry)?;
    }

    info!("graceful shutdown complete.");
    Ok(())
}
