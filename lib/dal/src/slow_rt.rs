use std::{
    future::Future,
    sync::OnceLock,
};

use si_runtime::main_tokio_runtime;
use thiserror::Error;
use tokio::{
    runtime::Runtime,
    task::JoinHandle,
};

/// A singleton for a second, alternative tokio runtime used for CPU intensive operations.
pub static SLOW_RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[derive(Debug, Error)]
pub enum SlowRuntimeError {
    #[error("io error: {0}")]
    Io(#[from] Box<std::io::Error>),
}

impl From<std::io::Error> for SlowRuntimeError {
    fn from(value: std::io::Error) -> Self {
        Box::new(value).into()
    }
}

pub type SlowRuntimeResult<T> = Result<T, SlowRuntimeError>;

/// Spawn a future onto the alternative "slow" runtime. Use this whenever you
/// need to perform an operation that could possibly take more than few
/// milliseconds between await points. This allows us to perform CPU intensive
/// operations without tying up the main tokio runtime, which needs to respond
/// immediately to network requests. The advantage to using a second Tokio
/// runtime instead of the blocking pool or a crate like rayon is that we can
/// use the same async code for these operations as everywhere else. The work
/// stealing behavior of the normal tokio runtime will work here as well, we
/// just don't care if some operations lock up the executor since we do not need
/// to respond immediately to network requests.
pub fn spawn<F>(future: F) -> SlowRuntimeResult<JoinHandle<F::Output>>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    Ok(match SLOW_RUNTIME.get() {
        Some(slow_rt) => slow_rt,
        None => {
            let slow_rt = main_tokio_runtime("slow_runtime")?;
            SLOW_RUNTIME.get_or_init(move || slow_rt)
        }
    }
    .spawn(future))
}
