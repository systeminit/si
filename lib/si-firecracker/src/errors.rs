use thiserror::Error;

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
    #[error("Mount error: {0}")]
    Mount(#[from] nix::Error),
    // Failed running a script to output
    #[error("Failed to run a script: {0}")]
    Output(String),
    // Failed to prepare a jail
    #[error("Failed to prepare a jail: {0}")]
    Prepare(#[source] tokio::io::Error),
    // Failed to setup firecracker
    #[error("Failed to setup firecracker: {0}")]
    Setup(#[from] tokio::io::Error),
    // Failed to spawn firecracker
    #[error("Failed to spawn firecracker: {0}")]
    Spawn(#[source] tokio::io::Error),
    // Failed to terminate firecracker
    #[error("Failed to terminate firecracker: {0}")]
    Terminate(#[from] cyclone_core::process::ShutdownError),
}
