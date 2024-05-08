use thiserror::Error;

/// Error type for [`PoolNoodle`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PoolNoodleError {
    /// Failed to get a new instance ID.
    #[error("Failed to get a new instance from the execution pool!")]
    ExecutionPoolStarved,
    /// Failed to clean an instance.
    #[error("Failed to clean the instance: {0}")]
    InstanceClean(String),
    /// Failed to prepare a new instance.
    #[error("Failed to prepare the instance: {0}")]
    InstancePrepare(String),
    /// Failed to spawn a new instance.
    #[error("Failed to spawn the instance: {0}")]
    InstanceSpawn(String),
    /// Failed to terminate an instance.
    #[error("Failed to terminate the instance: {0}")]
    InstanceTerminate(String),
}
