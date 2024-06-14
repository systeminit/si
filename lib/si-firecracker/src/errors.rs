use thiserror::Error;

/// Error type for [`PoolNoodle`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum FirecrackerJailError {
    // Failed to clean a jail
    #[error("Failed to clean a jail: {0}")]
    Clean(String),
    // Failed to prepare a jail
    #[error("Failed to prepare a jail: {0}")]
    Prepare(String),
    // Failed to setup firecracker
    #[error("Failed to setup firecracker: {0}")]
    Setup(String),
    // Failed to spawn firecracker
    #[error("Failed to spawn firecracker: {0}")]
    Spawn(String),
    // Failed to terminate firecracker
    #[error("Failed to terminate firecracker: {0}")]
    Terminate(#[from] cyclone_core::process::ShutdownError),
}
