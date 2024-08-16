use nix::unistd::{getpgrp, Pid};
#[cfg(target_os = "linux")]
use procfs::process::all_processes;
use std::collections::HashSet;
use std::result;
use telemetry::prelude::info;
use telemetry::tracing::debug;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use telemetry_utils::metric;
use thiserror::Error;
use tokio::time::sleep;

type Result<T> = result::Result<T, ProcessGathererError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ProcessGathererError {
    #[error("failed to get processes: {0}")]
    Process(#[from] procfs::ProcError),
    #[error("shutdown error: {0}")]
    Shutdown(#[from] mpsc::error::SendError<CancellationToken>),
}

pub struct ProcessGathererTask {
    client: ProcessGatherer,
    shutdown_token: CancellationToken,
}

impl ProcessGathererTask {
    const NAME: &'static str = "ProcessGathererTask";

    fn create(client: ProcessGatherer, shutdown_token: CancellationToken) -> Result<Self> {
        Ok(Self {
            client,
            shutdown_token,
        })
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    debug!(task = Self::NAME, "received cancellation");
                    self.client.stop().await;
                    break;
                }
                _ = self.client.start() => {
                }
                else => {
                    break;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct ProcessGatherer {
    procs: Mutex<HashSet<String>>,
}

impl ProcessGatherer {
    pub fn new() -> Self {
        let procs = HashSet::new().into();
        Self { procs }
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            {
                let mut procs = self.procs.lock().await;
                for proc in all_processes()?.flatten() {
                    if let Ok(stat) = proc.stat() {
                        if Pid::from_raw(stat.pgrp) == getpgrp() {
                            procs.insert(stat.comm);
                        }
                    }
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
    pub async fn stop(&mut self) {
        for proc in self.procs.lock().await.iter() {
            metric!(counter.cyclone.process = 1, proc = proc);
            info!(counter.cyclone.process = 1, proc = proc);
        }
    }
}

#[must_use]
pub struct ProcessGathererShutdownGuard {
    shutdown_token: CancellationToken,
}

impl ProcessGathererShutdownGuard {
    pub async fn wait(self) -> Result<()> {
        self.shutdown_token.cancelled().await;
        Ok(())
    }
}

pub fn init(
    enable: bool,
    tracker: &TaskTracker,
    shutdown_token: CancellationToken,
) -> Result<ProcessGathererShutdownGuard> {
    let client = ProcessGatherer::new();

    let guard = ProcessGathererShutdownGuard {
        shutdown_token: shutdown_token.clone(),
    };
    if enable {
        tracker.spawn(ProcessGathererTask::create(client, shutdown_token.clone())?.run());
    }

    Ok(guard)
}
