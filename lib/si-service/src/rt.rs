//! Common Tokio runtime related behavior.

use std::future::Future;

use color_eyre::{eyre::eyre, Result};
use si_runtime::DEFAULT_TOKIO_RT_THREAD_STACK_SIZE;
use tokio::runtime::Builder;

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

    let thread_builder =
        ::std::thread::Builder::new().stack_size(DEFAULT_TOKIO_RT_THREAD_STACK_SIZE);
    let thread_handler =
        thread_builder.spawn(|| si_runtime::main_tokio_runtime(thread_name)?.block_on(future))?;

    match thread_handler.join() {
        Ok(result) => result,
        Err(_) => Err(eyre!("couldn't join on the associated thread")),
    }
}

/// Create a customized Tokio runtime and block on a primary async function, i.e. an "async_main()".
///
/// # Notes
///
/// This function  creates a Tokio runtime on a spawned thread. It is intended
/// to be run as the entry point for a `main()` program as an alternative to the
/// `#[tokio::main]` attribute macro.
pub fn block_on_customize<S, Fut, F>(thread_name: S, future: Fut, f: F) -> Result<()>
where
    S: Into<String>,
    Fut: Future<Output = Result<()>> + Send + 'static,
    F: FnOnce(&mut Builder) + Send + 'static,
{
    let thread_name = thread_name.into();

    let thread_builder =
        ::std::thread::Builder::new().stack_size(DEFAULT_TOKIO_RT_THREAD_STACK_SIZE);
    let thread_handler = thread_builder
        .spawn(|| si_runtime::main_tokio_runtime_customize(thread_name, f)?.block_on(future))?;

    match thread_handler.join() {
        Ok(result) => result,
        Err(_) => Err(eyre!("couldn't join on the associated thread")),
    }
}
