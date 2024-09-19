use thiserror::Error;

/// Error type for [`PoolNoodle`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PoolNoodleError<E> {
    /// Failed to get a new instance ID.
    #[error("Failed to get a new instance from the execution pool!")]
    ExecutionPoolStarved,
    /// Failed to clean an instance.
    #[error("Failed to clean the instance: {0}")]
    InstanceClean(#[source] E),
    /// No instance found in task.
    #[error("No instance found in task.")]
    InstanceNotFound,
    /// Failed to prepare a new instance.
    #[error("Failed to prepare the instance: {0}")]
    InstancePrepare(#[source] E),
    /// Failed to spawn a new instance.
    #[error("Failed to spawn the instance: {0}")]
    InstanceSpawn(#[source] E),
    /// Failed to terminate an instance.
    #[error("Failed to terminate the instance: {0}")]
    InstanceTerminate(#[source] E),
    /// Failed to healthcheck instance creation.
    #[error("Failed to check pool health: {0}")]
    Unhealthy(#[source] E),
    /// Failed to healthcheck instance creation in time.
    #[error("Failed to check pool health in time")]
    UnhealthyTimeout(#[source] tokio::time::error::Elapsed),
}
