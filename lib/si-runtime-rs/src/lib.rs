//! Common Tokio runtime related behavior.

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use tokio::runtime::{Builder, Runtime};

pub const DEFAULT_TOKIO_RT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;
#[cfg(target_os = "linux")]
pub const DEFAULT_TOKIO_RT_BLOCKING_POOL_SIZE: usize = 512;
#[cfg(target_os = "macos")]
pub const DEFAULT_TOKIO_RT_BLOCKING_POOL_SIZE: usize = 16;

// Thread priority for compute executors (min = 0, max = 99, default = 50)
const COMPUTE_EXECUTOR_THREAD_PRIORITY: u8 = 25;
// Tokio runtime shutdown timeout for compute executors
const COMPUTE_EXECUTOR_TOKIO_RT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 10);

pub use tokio_dedicated_executor::{
    DedicatedExecutor, DedicatedExecutorError, DedicatedExecutorInitializeError,
    DedicatedExecutorJoinError,
};

/// Builds a main/primary Tokio [`Runtime`] with sensible defaults.
pub fn main_tokio_runtime(runtime_name: impl Into<String>) -> std::io::Result<Runtime> {
    main_tokio_runtime_customize(runtime_name, |_| {})
}

/// Builds a main/primary Tokio [`Runtime`] with sensible defaults and the ability to customize
pub fn main_tokio_runtime_customize<F>(
    runtime_name: impl Into<String>,
    f: F,
) -> std::io::Result<Runtime>
where
    F: FnOnce(&mut Builder),
{
    let mut builder = common_tokio_builder("main", runtime_name);
    builder
        .thread_stack_size(DEFAULT_TOKIO_RT_THREAD_STACK_SIZE)
        .max_blocking_threads(DEFAULT_TOKIO_RT_BLOCKING_POOL_SIZE)
        // Enables using net, process, signal, and some I/O types
        .enable_io();
    f(&mut builder);
    builder.build()
}

/// Builds a "compute" [`DedicatedExecutor`] for running CPU-intensive tasks.
pub fn compute_executor(name: &str) -> Result<DedicatedExecutor, DedicatedExecutorInitializeError> {
    DedicatedExecutor::new(
        format!("{name}-compute").as_str(),
        compute_tokio_builder(name),
        COMPUTE_EXECUTOR_THREAD_PRIORITY,
        COMPUTE_EXECUTOR_TOKIO_RT_SHUTDOWN_TIMEOUT,
    )
}

fn compute_tokio_builder(runtime_name: impl Into<String>) -> Builder {
    // NOTE: importantly this runtime does not have `enable_io()` turned on
    common_tokio_builder("compute", runtime_name)
}

fn common_tokio_builder(category: &'static str, runtime_name: impl Into<String>) -> Builder {
    let runtime_name = runtime_name.into();

    let mut builder = Builder::new_multi_thread();
    builder
        .thread_name_fn(move || {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            format!(
                "{}-tokio-{}-{}",
                category,
                runtime_name,
                ATOMIC_ID.fetch_add(1, Ordering::SeqCst)
            )
        })
        // Enables using `tokio::time`
        .enable_time();

    builder
}
