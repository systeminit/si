//! Common Tokio runtime related behavior.

use std::future::Future;

use color_eyre::{eyre::eyre, Result};

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

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
    let thread_handler = thread_builder.spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
            .thread_name(thread_name)
            .enable_all()
            .build()?
            .block_on(future)
    })?;

    match thread_handler.join() {
        Ok(result) => result,
        Err(_) => Err(eyre!("couldn't join on the associated thread")),
    }
}
