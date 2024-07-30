//! Common Tokio runtime related behavior.

use tokio::runtime::Runtime;

pub const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;
pub const RT_DEFAULT_BLOCKING_POOL_SIZE: usize = 2048; // this is 4x the tokio default of 512

/// Build a tokio runtime with sensible defaults
pub fn build_runtime<S>(thread_name: S) -> std::io::Result<Runtime>
where
    S: Into<String>,
{
    tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
        .thread_name(thread_name)
        .max_blocking_threads(RT_DEFAULT_BLOCKING_POOL_SIZE)
        .enable_all()
        .build()
}
