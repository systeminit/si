use core::panic;
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use tokio::runtime;

thread_local! {
    /// Parent Tokio [`runtime::Handle`] of a dedicated exectuor to be used for spawning tasks that
    /// are not intended to run on the executor runtime. See [`spawn_on_parent`]
    pub static EXECUTOR_PARENT_RUNTIME: RefCell<Option<runtime::Handle>> =
        const { RefCell::new(None) };
}

/// Registers the Tokio [`runtime::Handle`] as the parent Tokio runtime for this thread.
///
/// See: [`spawn_on_parent`]
pub fn register_parent_runtime(handle: Option<runtime::Handle>) {
    EXECUTOR_PARENT_RUNTIME.set(handle)
}

/// Registers current Tokio runtime [`runtime::Handle`] as the parent Tokio runtime for this
/// thread.
///
/// NOTE: this is primarily useful for testing.
pub fn register_current_runtime_as_parent() {
    register_parent_runtime(Some(runtime::Handle::current()));
}

/// Runs a [`Future`] on the Tokio runtime registered by [`register_parent_runtime`].
///
/// # Panic
///
/// A parent Tokio runtime must be [registered](register_parent_runtime) before calling this
/// function.
pub async fn spawn_on_parent<Fut>(fut: Fut) -> Fut::Output
where
    Fut: Future + Send + 'static,
    Fut::Output: Send,
{
    let parent_tokio_rt_handle = EXECUTOR_PARENT_RUNTIME
        .with_borrow(|handle| handle.clone())
        .expect(concat!(
            "no parent runtime registered; ",
            "`register_parent_runtime()` or `register_current_runtime_as_parent()` ",
            "must be called in current thread",
        ));
    DropGuard(parent_tokio_rt_handle.spawn(fut)).await
}

struct DropGuard<T>(tokio::task::JoinHandle<T>);

impl<T> Future for DropGuard<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[allow(clippy::panic)] // the panic may only result from improper runtime shutdowns
        Poll::Ready(match std::task::ready!(self.0.poll_unpin(cx)) {
            Ok(value) => value,
            Err(err) if err.is_cancelled() => panic!("parent runtime was shut down"),
            Err(err) => std::panic::resume_unwind(err.into_panic()),
        })
    }
}

impl<T> Drop for DropGuard<T> {
    fn drop(&mut self) {
        self.0.abort()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn happy_path() {
        let parent_tokio_rt = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("failed to build parent runtime");

        let parent_tokio_rt_thread_id = parent_tokio_rt
            .spawn(async move { std::thread::current().id() })
            .await
            .expect("task failed to join");
        let parent_thread_id = std::thread::current().id();
        assert_ne!(parent_tokio_rt_thread_id, parent_thread_id);

        register_parent_runtime(Some(parent_tokio_rt.handle().clone()));

        let measured_thread_id = spawn_on_parent(async move { std::thread::current().id() }).await;
        assert_eq!(measured_thread_id, parent_tokio_rt_thread_id);

        parent_tokio_rt.shutdown_background();
    }

    #[tokio::test]
    #[should_panic(expected = "no parent runtime registered")]
    async fn panic_if_no_runtime_registered() {
        spawn_on_parent(futures::future::ready(())).await;
    }

    #[tokio::test]
    #[should_panic(expected = "parent runtime was shut down")]
    async fn panic_if_io_runtime_down() {
        let parent_tokio_rt_io = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("failed to build parent runtime");

        register_parent_runtime(Some(parent_tokio_rt_io.handle().clone()));

        tokio::task::spawn_blocking(move || {
            parent_tokio_rt_io.shutdown_timeout(Duration::from_secs(1));
        })
        .await
        .expect("task failed to join");

        spawn_on_parent(futures::future::ready(())).await;
    }
}
