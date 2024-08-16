//! Service/server start pre-processing for establishing/reigstering service:
//! - Health [to come]
//! - Version
//! - Anything else to do async or sync during service startup

use glob::glob;
use std::env;
use std::io;
use std::path::Component;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    fs::File, 
    io::AsyncReadExt,
    signal::unix::{self, SignalKind},
    sync::{mpsc, oneshot},
};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use std::result;
use tokio::time::Instant;
pub use telemetry::{ApplicationTelemetryClient, TelemetryClient};
use std::thread;
use tokio::time::Duration;
use tokio::time;

/// An error that can be returned when starting the process for the binary
#[derive(Debug, Error)]
pub enum StartupError {
    /// When the version could not be established
    #[error("Failed to establish version: {0}")]
    Signal(#[source] io::Error),
}

/// A management client which holds handles to a processes management
#[derive(Clone, Debug)]
pub struct ApplicationManagementClient {
    update_management_tx: mpsc::UnboundedSender<ManagementCommand>,
}

impl ApplicationManagementClient {
    pub fn new(
        update_management_tx: mpsc::UnboundedSender<ManagementCommand>,
    ) -> Self { 
        Self {
            update_management_tx,
        }
    }
}

pub enum ClientError {
    SendError(tokio::sync::mpsc::error::SendError<ManagementCommand>),
    // other variants...
}

#[derive(Debug, Error)]
pub enum ManagementSignalError {
    #[error("Signal error: {0}")]
    Signal(io::Error),
}

type ManagementResult<T> = result::Result<T, ManagementSignalError>;

struct ManagementSignalHandlerTask {
    client: ApplicationManagementClient,
    shutdown_token: CancellationToken,
    sig_usr2: unix::Signal,
}

impl ManagementSignalHandlerTask {
    const NAME: &'static str = "ManagementSignalHandlerTask";

    fn create(
        client: ApplicationManagementClient,
        shutdown_token: CancellationToken,
    ) -> io::Result<Self> {
        let sig_usr2 = unix::signal(SignalKind::user_defined2())?;

        Ok(Self {
            client,
            shutdown_token,
            sig_usr2,
        })
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    debug!(task = Self::NAME, "received cancellation");
                    break;
                }
                Some(_) = self.sig_usr2.recv() => {
                    let applicationRuntimeFlag = "FLICKING THE MAINTENANCE MODE SWITCH";
                    dbg!(&applicationRuntimeFlag);
                }
                else => {
                    // All other arms are closed, nothing let to do but return
                    trace!(task = Self::NAME, "all signal listeners have closed");
                    break;
                }
            }
        }

        debug!(task = Self::NAME, "shutdown complete");
    }
}

struct ManagementUpdateTask {
    update_command_rx: mpsc::UnboundedReceiver<ManagementCommand>,
    is_shutdown: bool,
}

impl ManagementUpdateTask {
    const NAME: &'static str = "ManagementUpdateTask";

    fn new(
        update_command_rx: mpsc::UnboundedReceiver<ManagementCommand>,
    ) -> Self {
        Self {
            update_command_rx,
            is_shutdown: false,
        }
    }

    async fn run(mut self) {
        while let Some(command) = self.update_command_rx.recv().await {
            match command {
                ManagementCommand::RunTimeMode { wait } => {
                    dbg!("Hello there friends, from your tracing level management command");
                    if let Some(tx) = wait {
                        if let Err(err) = tx.send(()) {
                            warn!(
                                error = ?err,
                                "receiver already closed when waiting on changing application runtime mode",
                            );
                        }
                    }
                }
                ManagementCommand::Shutdown(token) => {
                    if !self.is_shutdown {
                        Self::shutdown().await;
                    }
                    self.is_shutdown = true;
                    token.cancel();
                    break;
                }
            }
        }

        debug!(task = Self::NAME, "shutdown complete");
    }

    async fn shutdown() {

        let (tx, wait_on_shutdown) = oneshot::channel();

        let started_at = Instant::now();
        let _ = thread::spawn(move || {
            // Take some action here to close the management thread down
            tx.send(()).ok();
        });

        let timeout = Duration::from_secs(5);
        match time::timeout(timeout, wait_on_shutdown).await {
            Ok(Ok(_)) => debug!(
                time_ns = (Instant::now() - started_at).as_nanos(),
                "management thread shutdown"
            ),
            Ok(Err(_)) => trace!("management thread shutdown sender already closed"),
            Err(_elapsed) => {
                warn!(
                    ?timeout,
                    "management thread shutdown took too long, not waiting for full shutdown"
                );
            }
        };
    }

}

// Create Management Client for Service Management
fn create_client(
    tracker: &TaskTracker,
    shutdown_token: CancellationToken,

) -> ManagementResult<(ApplicationManagementClient, ManagementShutdownGuard)> {

    let (update_management_tx, update_management_rx) = mpsc::unbounded_channel();

    let client = ApplicationManagementClient::new(
        update_management_tx.clone()
    );

    let guard = ManagementShutdownGuard {
        update_management_tx,
    };

    // Spawn this task free of the tracker as we want it to outlive the tracker when shutting down
    tokio::spawn(ManagementUpdateTask::new(update_management_rx).run());

    // This might need to be behind some kind of if?
    tracker.spawn(
        ManagementSignalHandlerTask::create(client.clone(), shutdown_token.clone())
            .map_err(ManagementSignalError::Signal)?
            .run(),
    );

    Ok((client, guard))
}

pub fn init(
    tracker: &TaskTracker,
    shutdown_token: CancellationToken,
) -> ManagementResult<(ApplicationManagementClient, ManagementShutdownGuard)> {
    let (client, guard) = create_client(tracker, shutdown_token)?;
    Ok((client, guard))
}

pub struct ManagementShutdownGuard {
    update_management_tx: mpsc::UnboundedSender<ManagementCommand>,
}

impl ManagementShutdownGuard {
    pub async fn wait(self) -> result::Result<(), ClientError> {
        let token = CancellationToken::new();
        self.update_management_tx
            .send(ManagementCommand::Shutdown(token.clone()))
            .map_err(|err| ClientError::SendError(err))?;
        token.cancelled().await;
        Ok(())
    }
}

pub enum ManagementCommand {
    Shutdown(CancellationToken),
    RunTimeMode {
        wait: Option<oneshot::Sender<()>>,
    },
}

/// Gracefully start a service and conduct pre-processing of service handler
pub async fn startup(service: &str) -> Result<(), std::io::Error> {
    let executable_path = match env::current_exe() {
        Ok(exe_path) => exe_path,
        Err(_) => {
            info!(
                "could not establish running executable path for {}",
                service
            );
            return Ok(());
        }
    };

    let executable_path = match executable_path.canonicalize() {
        Ok(exe_path) => exe_path,
        Err(_) => {
            info!("could not canonicalize executable path for {}", service);
            return Ok(());
        }
    };

    // Check if it's a dev build (i.e. running from buck-out)
    if executable_path
        .components()
        .any(|path| Component::Normal("buck-out".as_ref()) == path)
    {
        debug!(
            "development build (buck) detected for {}, no metadata can be reported",
            service
        );
        return Ok(());
    }

    let metadata_candidates = match glob(&format!("/etc/nix-omnibus/{}/*/metadata.json", service)) {
        Ok(iter) => iter,
        Err(_) => {
            info!("metadata candidates could not be found for {}", service);
            return Ok(());
        }
    };

    let mut metadata_candidates = match metadata_candidates.collect::<Result<Vec<_>, _>>() {
        Ok(vec) => vec,
        Err(_) => {
            info!(
                "could not collect PathBufs from metadata_candidates for {}",
                service
            );
            return Ok(());
        }
    };

    // Sort them lexically so that the latest (if there is more than one) is at the bottom
    // There is a minor issue here if we `downgrade` a single running server/host there is the potential
    // that this reports the newer version, rather than the rollback version due to how we are
    // lexically sorting. At the time of writing there was no viable method to determine which exact
    // metadata file should be referenced.
    metadata_candidates.sort();

    // Take the last one (the latest)
    let metadata_file_path = match metadata_candidates.pop() {
        Some(file_path) => file_path,
        None => {
            info!(
                "could not read appropriate metadata files for {} to determine version",
                service
            );
            return Ok(());
        }
    };

    // Read contents of metadata file
    let mut metadata_file_handler = match File::open(&metadata_file_path).await {
        Ok(metadata_file_handler) => metadata_file_handler,
        Err(_) => {
            info!("metadata file could not be read for {}", service);
            return Ok(());
        }
    };

    let mut file_contents = String::new();
    metadata_file_handler
        .read_to_string(&mut file_contents)
        .await?;

    let metadata_file_path_str = metadata_file_path.as_path().display().to_string();

    info!(file_contents, metadata_file_path_str, "metadata contents:");

    Ok(())
}
