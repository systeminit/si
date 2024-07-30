//! Common Tokio runtime related behavior.

use std::future::Future;

use color_eyre::{eyre::eyre, Result};
use tokio::runtime::Runtime;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;
const RT_DEFAULT_BLOCKING_POOL_SIZE: usize = 2048; // this is 4x the tokio default of 512

/// Create a Tokio runtime and block on a primary async function, i.e. an "async_main()".
///
/// # Notes
///
/// This funciton  creates a Tokio runtime on a spawned thread. It is intended to be run as the
/// entry point for a `main()` program as an alternative to the `#[tokio::main]` attribute macro.
pub fn block_on<S, Fut>(thread_name: S, future: Fut) -> Result<()>
where
    S: Into<String>,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let thread_name = thread_name.into();

    let thread_builder = ::std::thread::Builder::new().stack_size(RT_DEFAULT_THREAD_STACK_SIZE);
    let thread_handler = thread_builder.spawn(|| build_runtime(thread_name)?.block_on(future))?;

    match thread_handler.join() {
        Ok(result) => result,
        Err(_) => Err(eyre!("couldn't join on the associated thread")),
    }
}

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
