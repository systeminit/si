use std::{io, num::TryFromIntError, process::ExitStatus, time::Duration};

use nix::{sys::signal, unistd::Pid};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{process::Child, time};

pub use nix::sys::signal::Signal;

const CHILD_WAIT_TIMEOUT_SECS: Duration = Duration::from_secs(10);

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ShutdownError {
    #[error("failed to wait on child process")]
    ChildWait(#[source] io::Error),
    #[error("failed to convert pid from u32 to i32--overflow")]
    PidOverflow(#[from] TryFromIntError),
    #[error("failed to signal child")]
    Signal(#[from] nix::errno::Errno),
    #[error("sending SIGKILL failed")]
    StartKill(#[source] io::Error),
}

pub async fn child_shutdown(
    child: &mut Child,
    signal: Option<Signal>,
    wait_timeout: Option<Duration>,
) -> Result<ExitStatus, ShutdownError> {
    if let (Some(signal), Some(pid)) = (signal, child.id()) {
        trace!("sending {} to child process {}", signal, pid);
        // Thanks, Clippy!
        // See: https://rust-lang.github.io/rust-clippy/master/index.html#cast_possible_wrap
        let pid = i32::try_from(pid)?;
        signal::kill(Pid::from_raw(pid), signal)?;
    }

    match time::timeout(
        wait_timeout.unwrap_or(CHILD_WAIT_TIMEOUT_SECS),
        child.wait(),
    )
    .await
    {
        Ok(wait_result) => {
            let exit_status = wait_result.map_err(ShutdownError::ChildWait)?;
            if !exit_status.success() {
                warn!("child process had a nonzero exit; code={}", exit_status);
            }

            Ok(exit_status)
        }
        Err(_elapsed) => {
            child.start_kill().map_err(ShutdownError::StartKill)?;
            let exit_status = child.wait().await.map_err(ShutdownError::ChildWait)?;
            if !exit_status.success() {
                warn!("child process had a nonzero exit; code={}", exit_status);
            }

            Ok(exit_status)
        }
    }
}
