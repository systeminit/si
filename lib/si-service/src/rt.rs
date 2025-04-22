//! Common Tokio runtime related behavior.

use std::future::Future;

use color_eyre::{
    Result,
    eyre::eyre,
};
use si_runtime::{
    CoreId,
    DEFAULT_TOKIO_RT_THREAD_STACK_SIZE,
};

/// Create a Tokio runtime and block on a primary async function, i.e. an "async_main()".
///
/// # Notes
///
/// This function  creates a Tokio runtime on a spawned thread. It is intended
/// to be run as the entry point for a `main()` program as an alternative to the
/// `#[tokio::main]` attribute macro.
pub fn block_on<S, Fut>(thread_name: S, future: Fut) -> Result<()>
where
    S: Into<String>,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let thread_name = thread_name.into();

    inner_block_on(|| si_runtime::main_tokio_runtime(thread_name)?.block_on(future))
}

/// Create a Tokio runtime with worker threads pinned to CPU cores and block on a primary async
/// function, i.e. an "async_main()".
///
/// # Notes
///
/// This function  creates a Tokio runtime on a spawned thread. It is intended
/// to be run as the entry point for a `main()` program as an alternative to the
/// `#[tokio::main]` attribute macro.
pub fn block_on_with_core_affinity<S, Fut>(
    thread_name: S,
    future: Fut,
    cpu_cores: Vec<CoreId>,
) -> Result<()>
where
    S: Into<String>,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let thread_name = thread_name.into();

    inner_block_on(|| {
        si_runtime::main_tokio_runtime_with_core_affinitiy(thread_name, cpu_cores)?.block_on(future)
    })
}

#[inline]
fn inner_block_on<F>(thread_fn: F) -> Result<()>
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    let thread_builder =
        ::std::thread::Builder::new().stack_size(DEFAULT_TOKIO_RT_THREAD_STACK_SIZE);
    let thread_handler = thread_builder.spawn(thread_fn)?;

    match thread_handler.join() {
        Ok(result) => result,
        Err(_) => Err(eyre!("couldn't join on the associated thread")),
    }
}
