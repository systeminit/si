//! Graceful service/server shutdown using cancellation tokens, task trackers, driven by Unix
//! signal handling.

use std::{
    convert::Infallible,
    error,
    future::Future,
    io,
    time::Duration,
};

use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix::{
        self,
        SignalKind,
    },
    task::JoinHandle,
    time,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

/// An error that can be returned when gracefully shutting down.
///
/// See [`graceful`] for more details.
#[derive(Debug, Error)]
pub enum ShutdownError {
    /// When a main handle returns an error
    #[error("main handle returned an error: {0}")]
    Handle(#[source] Box<dyn error::Error + Send + Sync + 'static>),
    /// When a main handle returns a join error
    #[error("main handle returned a join error")]
    Join,
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
/// This facility sets up a signal handler for both `SIGINT` (i.e. `Ctrl+c`) and `SIGTERM` so usage
/// of this function with other code intercepting these signals is *highly* discouraged.
pub fn graceful<TelemetryFut, E>() -> GracefulShutdown<TelemetryFut, Infallible>
where
    TelemetryFut: Future<Output = Result<(), E>>,
    E: error::Error + Send + Sync + 'static,
{
    GracefulShutdown::default()
}

/// Gracfully shutdown a service with a "main" handle that may be running multiple tasks and
/// in-flight work.
///
/// # Platform-specific behavior
///
/// This facility sets up a signal handler for both `SIGINT` (i.e. `Ctrl+c`) and `SIGTERM` so usage
/// of this function with other code intercepting these signals is *highly* discouraged.
pub fn graceful_with_handle<TelemetryFut, E, HanErr>(
    handle: JoinHandle<Result<(), HanErr>>,
) -> GracefulShutdown<TelemetryFut, HanErr>
where
    TelemetryFut: Future<Output = Result<(), E>>,
    E: error::Error + Send + Sync + 'static,
{
    GracefulShutdown {
        main_handle: Some(handle),
        ..Default::default()
    }
}

/// Constructs and performs a graceful shutdown.
#[derive(Debug)]
pub struct GracefulShutdown<TelemetryFut, HanErr> {
    main_handle: Option<JoinHandle<Result<(), HanErr>>>,
    groups: Vec<(TaskTracker, CancellationToken)>,
    telemetry_guard: Option<TelemetryFut>,
    timeout: Option<Duration>,
}

impl<TelemetryFut, E, HanErr> Default for GracefulShutdown<TelemetryFut, HanErr>
where
    TelemetryFut: Future<Output = Result<(), E>>,
    E: error::Error + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            main_handle: Default::default(),
            groups: Default::default(),
            telemetry_guard: Default::default(),
            timeout: Default::default(),
        }
    }
}

impl<TelemetryFut, E, HanErr> GracefulShutdown<TelemetryFut, HanErr>
where
    TelemetryFut: Future<Output = Result<(), E>>,
    E: error::Error + Send + Sync + 'static,
    HanErr: error::Error + Send + Sync + 'static,
{
    /// Adds a shutdown group, consisting of a related [`TaskTracker`] and [`CancellationToken`].
    pub fn group(mut self, tracker: TaskTracker, token: CancellationToken) -> Self {
        self.groups.extend([(tracker, token)]);
        self
    }

    /// Adds a collection of shutdown groups, consisting of a related [`TaskTracker`] and
    /// [`CancellationToken`].
    pub fn groups<I>(mut self, shutdown_groups: I) -> Self
    where
        I: IntoIterator<Item = (TaskTracker, CancellationToken)>,
    {
        self.groups.extend(shutdown_groups);
        self
    }

    /// Adds a telemetry shutdown guard.
    pub fn telemetry_guard(mut self, telemetry_guard: TelemetryFut) -> Self {
        self.telemetry_guard = Some(telemetry_guard);
        self
    }

    /// Clears any prior set telemetry shutdown guard.
    pub fn clear_telemetry_guard(mut self) -> Self {
        self.telemetry_guard.take();
        self
    }

    /// Adds a graceful shutdown timeout duration, after which graceful shutdown will cancel.
    pub fn timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.timeout = timeout.into();
        self
    }

    /// Waits until all graceful shutdown conditions have been met.
    ///
    /// # Platform-specific behavior
    ///
    /// This function sets up a signal handler for both `SIGINT` (i.e. `Ctrl+c`) and `SIGTERM` so
    /// usage of this function with other code intercepting these signals is *highly* discouraged.
    pub async fn wait(self) -> Result<(), ShutdownError> {
        let Self {
            main_handle,
            groups,
            telemetry_guard,
            timeout,
        } = self;

        let mut sig_int = unix::signal(SignalKind::interrupt()).map_err(ShutdownError::Signal)?;
        let mut sig_term = unix::signal(SignalKind::terminate()).map_err(ShutdownError::Signal)?;

        let maybe_handle_result = match main_handle {
            Some(main_handle) => {
                tokio::select! {
                    join_result = main_handle => {
                        trace!("main handle completed");
                        match join_result {
                            Ok(result) => Some(result.map_err(|err| {
                                ShutdownError::Handle(Box::new(err))
                            })),
                            Err(_join_err) => Some(Err(ShutdownError::Join)),
                        }
                    }
                    _ = sig_int.recv() => {
                        info!("received SIGINT, performing graceful shutdown");
                        None
                    }
                    _ = sig_term.recv() => {
                        info!("received SIGTERM, performing graceful shutdown");
                        None
                    }
                }
            }
            None => {
                tokio::select! {
                    _ = sig_int.recv() => {
                        info!("received SIGINT, performing graceful shutdown");
                        None
                    }
                    _ = sig_term.recv() => {
                        info!("received SIGTERM, performing graceful shutdown");
                        None
                    }
                }
            }
        };

        let total = groups.len();
        let mut current: usize = 1;

        let await_groups = async {
            for (tracker, token) in groups {
                debug!("performing graceful shutdown for group(s) {current}/{total}");
                tracker.close();
                token.cancel();
                tracker.wait().await;
                current = current.saturating_add(1);
            }
        };

        // Wait for all tasks to finish
        match timeout {
            Some(timeout) => {
                if let Err(_elapsed) = time::timeout(timeout, await_groups).await {
                    warn!("graceful shutdown timeout exceeded; completing shutdown anyway");
                    if let Some(telemetry_guard) = telemetry_guard {
                        warn!("performing graceful shutdown for telemetry guard");
                        telemetry_guard.await.map_err(ShutdownError::telemetry)?;
                    }
                    return Err(ShutdownError::TimeoutElapsed(timeout));
                }
            }
            None => {
                await_groups.await;
            }
        }

        if let Some(telemetry_guard) = telemetry_guard {
            debug!("performing graceful shutdown for telemetry guard");
            telemetry_guard.await.map_err(ShutdownError::telemetry)?;
        }

        info!("graceful shutdown complete.");
        match maybe_handle_result {
            Some(handle_result) => handle_result,
            None => Ok(()),
        }
    }
}
