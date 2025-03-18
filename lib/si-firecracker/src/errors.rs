use std::path::PathBuf;

use thiserror::Error;

use crate::stream::StreamForwarderError;

/// Error type for [`PoolNoodle`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum FirecrackerJailError {
    // Failed to clean a jail
    #[error("Failed to clean a jail: {0}")]
    Clean(#[source] tokio::io::Error),
    // Error from DmSetup
    #[error("dmsetup error: {0}")]
    DmSetup(#[from] devicemapper::DmError),
    // Failed to interact with a mount
    #[error("Mount error when working with {1}: {0}")]
    Mount(#[source] nix::Error, PathBuf),
    // Failed running a script to output
    #[error("Failed to run a script: {0}")]
    Output(String),
    // Failed to prepare a jail
    #[error("Failed to prepare a jail: {0}")]
    Prepare(#[source] tokio::io::Error),
    // Failed to setup firecracker
    #[error("Failed to setup firecracker: {0}")]
    Setup(#[from] tokio::io::Error),
    // The setup script(s) do not exist
    #[error("Setup script(s) do not exist: {0:?}")]
    SetupScriptsDoNotExist(Vec<String>),
    // Failed to spawn firecracker
    #[error("Failed to spawn firecracker: {0}")]
    Spawn(#[source] tokio::io::Error),
    // StreamForwarderError
    #[error("StreamForwarderError: {0}")]
    StreamForwarder(#[from] StreamForwarderError),
    // Failed to terminate firecracker
    #[error("Failed to terminate firecracker: {0}")]
    Terminate(#[from] cyclone_core::process::ShutdownError),
}
