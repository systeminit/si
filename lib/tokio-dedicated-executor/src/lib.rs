//! An executor which manages a Tokio runtime that is dedicated to a specific set of workloads. The
//! futures and any spawned tasks will run on this runtime which can be purpose-tuned.
//!
//! The implementation of this crate comes from the [`executor`] crate in [InfluxData]'s
//! [influxdb3_core] project which is collectively released under the [MIT] or [Apache v2.0]
//! license.
//!
//! This implementation is based on the `executor` crate as of July 10, 2024:
//!
//! <https://github.com/influxdata/influxdb3_core/tree/78b4d56989410b30a3cc48020c1491405943b4ad/executor>
//!
//! # References
//!
//! - [The New Stack: Using Rustlang’s Async Tokio Runtime for CPU-Bound Tasks](https://thenewstack.io/using-rustlangs-async-tokio-runtime-for-cpu-bound-tasks/)
//! - [Rustacean Station: Rebuilding InfluxDB with Rust with Andrew Lamb](https://rustacean-station.org/episode/andrew-lamb/)
//! - [`executor`] crate
//!
//! [Apache v2.0]: https://github.com/influxdata/influxdb3_core/blob/main/LICENSE-APACHE
//! [InfluxData]: https://www.influxdata.com/
//! [MIT]: https://github.com/influxdata/influxdb3_core/blob/main/LICENSE-MIT
//! [`executor`]: https://github.com/influxdata/influxdb3_core/tree/main/executor
//! [influxdb3_core]: https://github.com/influxdata/influxdb3_core

#![warn(
    clippy::unwrap_in_result,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    missing_docs
)]

use std::{
    fmt,
    future::Future,
    sync::Arc,
    thread,
    time::Duration,
};

use futures::{
    FutureExt,
    TryFutureExt,
    future::{
        BoxFuture,
        Shared,
    },
};
use parking_lot::RwLock;
use thiserror::Error;
use thread_priority::{
    ThreadPriority,
    set_current_thread_priority,
};
use tokio::{
    runtime,
    sync::oneshot,
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;
use tracing::warn;

mod parent;

pub use parent::{
    register_current_runtime_as_parent,
    register_parent_runtime,
    spawn_on_parent,
};

/// Runs futures (and any [`tokio::spawn`]ed tasks) on a seperate & dedicated Tokio runtime.
///
/// Such Tokio runtimes can be tuned for specific workloads, priorities, thread counts, etc.
///
/// # Task Scheduling
///
/// The work performed by this executor (and thus on the underlying Tokio runtime) may be
/// particular and specific, for example performing CPU-intensive work in an executor thereby
/// preventing the slow down of the main Tokio runtime. If such work requires tasks to be spawned
/// back on the original Tokio runtime (referred to here as the "parent" Tokio runtime), this can
/// be accomplished by using [`spawn_on_parent`].
#[derive(Clone)]
pub struct DedicatedExecutor {
    state: Arc<RwLock<State>>,
}

impl fmt::Debug for DedicatedExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DedicatedExecutor").finish_non_exhaustive()
    }
}

impl DedicatedExecutor {
    /// Creates a [`DedicatedExecutor`] for work that is seperate and runtime-isolated from any
    /// other Tokio runtimes.
    ///
    /// # Implementation Notes
    ///
    /// The implementation uses a techique found on Stack Overflow to create a new Tokio runtime
    /// which may have been invoked *within* an existing Tokio runtime.
    ///
    /// See: https://stackoverflow.com/a/62536772
    pub fn new(
        name: &str,
        mut tokio_rt_builder: runtime::Builder,
        thread_priority: impl Into<Option<u8>>,
        shutdown_timeout: Duration,
    ) -> Result<Self, DedicatedExecutorInitializeError> {
        let manaing_tokio_rt_thread_name = format!("{name}-dedicated-executor-manager");

        let shutdown_token = CancellationToken::new();
        let (shutdown_completed_tx, shutdown_completed_rx) = oneshot::channel();
        let (handle_result_tx, handle_result_rx) = std::sync::mpsc::channel();

        let parent_tokio_rt_handle = runtime::Handle::try_current().ok();

        let executor_shutdown_token = shutdown_token.clone();

        let maybe_thread_priority = match thread_priority.into() {
            Some(value) => Some(ThreadPriority::try_from(value).map_err(|_| {
                DedicatedExecutorInitializeError::new(ThreadPriorityParseError(value))
            })?),
            None => None,
        };

        let managing_tokio_rt_thread = thread::Builder::new()
            .name(manaing_tokio_rt_thread_name)
            .spawn(move || {
                // Register parent Tokio runtime for current thread
                parent::register_parent_runtime(parent_tokio_rt_handle.clone());

                #[allow(clippy::blocks_in_conditions)]
                let tokio_rt = match tokio_rt_builder
                    // Register parent Tokio runtime on new runtime threads
                    .on_thread_start(move || {
                        if let Some(thread_priority) = maybe_thread_priority {
                            if let Err(err) = set_current_thread_priority(thread_priority) {
                                warn!(
                                    error = ?err,
                                    ?thread_priority,
                                    "failed to set thread priority",
                                );
                            }
                        }

                        parent::register_parent_runtime(parent_tokio_rt_handle.clone());
                    })
                    .build()
                {
                    Ok(tokio_rt) => tokio_rt,
                    Err(err) => {
                        handle_result_tx.send(Err(err)).ok();
                        return; // Early return if we failed to build the Tokio runtime
                    }
                };

                tokio_rt.block_on(async move {
                    // Send the [`runtime::Handle`] back to the constructor's thread
                    if handle_result_tx
                        .send(Ok(runtime::Handle::current()))
                        .is_err()
                    {
                        return; // Early return if we failed to send back the handle
                    }

                    // Wait for a shutdown of the executor to be triggered
                    executor_shutdown_token.cancelled().await;
                });

                // Shutdown the Tokio runtime, waiting at most the given duration for all spawned
                // work to complete.
                tokio_rt.shutdown_timeout(shutdown_timeout);

                // Signal that executor shutdown has completed
                shutdown_completed_tx.send(()).ok();
            })
            // Failed to successfully spawn thread
            .map_err(DedicatedExecutorInitializeError::new)?;

        // Read the [`runtime::Handle`] from the managing thread
        let handle = handle_result_rx
            .recv()
            // RecvError
            .map_err(DedicatedExecutorInitializeError::new)?
            // Error from initializing inside thread
            .map_err(DedicatedExecutorInitializeError::new)?;

        let state = State {
            tokio_rt_handle: Some(handle),
            managing_tokio_rt_thread: Some(managing_tokio_rt_thread),
            shutdown_token,
            shutdown_completed: shutdown_completed_rx.map_err(Arc::new).boxed().shared(),
        };

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
        })
    }

    /// Runs the [`Future`] (and any tasks it spawns) on the `DedicatedExecutor`.
    ///
    /// # Cancellation
    ///
    /// If the returned `Future` is dropped then the task is immediately aborted.
    #[allow(clippy::missing_panics_doc)]
    pub fn spawn<T>(
        &self,
        task: T,
    ) -> impl Future<Output = Result<T::Output, DedicatedExecutorError>> + use<T>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let maybe_tokio_rt_handle = {
            let state = self.state.read();
            state.tokio_rt_handle.clone()
        };

        let Some(tokio_rt_handle) = maybe_tokio_rt_handle else {
            return futures::future::err(DedicatedExecutorError::WorkerGone).boxed();
        };

        // NOTE: we are using a [`JoinSet`] to benefit from its "cancel on drop" behavior:
        //
        // > When the JoinSet is dropped, all tasks in the JoinSet are immediately aborted.
        // See: https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html
        let mut join_set = JoinSet::new();
        join_set.spawn_on(task, &tokio_rt_handle);

        async move {
            #[allow(clippy::expect_used)] // task spawned & immediately joined
            join_set
                .join_next()
                .await
                .expect("just spawned task; will not be none")
                .map_err(|err| match err.try_into_panic() {
                    // Task had panicked
                    Ok(err) => {
                        let msg = if let Some(s) = err.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = err.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "unknown internal error".to_string()
                        };

                        DedicatedExecutorError::TaskPanicked(msg)
                    }
                    // Not a panic, runtime has likely shut down
                    Err(_) => DedicatedExecutorError::WorkerGone,
                })
        }
        .boxed()
    }

    /// Triggers the shutdown of this executor and any clones.
    pub fn shutdown(&self) {
        let mut state = self.state.write();
        state.tokio_rt_handle.take();
        // Trigger the managing Tokio runtime's thread to shut down
        state.shutdown_token.cancel();
    }

    /// Shuts down the executor and any clones.
    ///
    /// All subsequent tasks executions are stopped and the managing thread is await for its
    /// completion.
    ///
    /// NOTE: all clones of this `DedicatedExecutor` will be shut down as well.
    ///
    /// # Implementation Notes
    ///
    /// Only the first call to `join` will wait for the managing thread to complete. All subsequent
    /// calls to `join` will complete immediately.
    ///
    /// # Panic
    ///
    /// [`DedicatedExecutor`] implements shutdown on [`Drop`] (indirectly through dropping its
    /// internal state). You should rely on this behavior and *not* call `join` manually during
    /// [`Drop`] or panics as this may lead to another panic. For more detail, see:
    /// <https://github.com/rust-lang/futures-rs/issues/2575>.
    pub async fn join(&self) -> Result<(), DedicatedExecutorJoinError> {
        self.shutdown();

        let shutdown_completed = {
            let state = self.state.read();
            state.shutdown_completed.clone()
        };

        shutdown_completed
            .await
            .map_err(|_| DedicatedExecutorJoinError)
    }
}

/// Error when running a spawned [`DedicatedExecutor`] task.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum DedicatedExecutorError {
    /// When a task panics
    #[error("task panicked: {0}")]
    TaskPanicked(String),
    /// When attempting to spawn a task and the executor has already shut down
    #[error("worker thread is gone, executor has likely shut down")]
    WorkerGone,
}

/// Error when initializing a [`DedicatedExecutor`].
#[derive(Debug, Error)]
#[error("failed to initialize executor: {0}")]
pub struct DedicatedExecutorInitializeError(
    #[source] Box<dyn std::error::Error + 'static + Sync + Send>,
);

impl DedicatedExecutorInitializeError {
    /// Creates a new `DedicatedExecutorInitializeError`.
    pub fn new<E>(err: E) -> Self
    where
        E: std::error::Error + 'static + Sync + Send,
    {
        Self(Box::new(err))
    }
}

/// Error when calling [`DedicatedExecutor::join`].
#[derive(Debug, Error)]
#[error("error while awaiting shutdown; sender already closed")]
pub struct DedicatedExecutorJoinError;

/// Error when parsing a thread priority value
#[derive(Debug, Error)]
#[error("failed parse thread priority value: {0}")]
struct ThreadPriorityParseError(u8);

/// Interior state for [`DedicatedExecutor`].
struct State {
    /// Tokio Runtime handle.
    ///
    /// NOTE: value is `None` when runtime is shutting down.
    tokio_rt_handle: Option<runtime::Handle>,
    /// Managing thread is managing the Tokio runtime and can be joined during [`Drop`].
    managing_tokio_rt_thread: Option<thread::JoinHandle<()>>,
    /// Token that when triggered will initiate a executor shutdown.
    shutdown_token: CancellationToken,
    /// Future that when ready signals that shutdown has completed.
    shutdown_completed: Shared<BoxFuture<'static, Result<(), Arc<oneshot::error::RecvError>>>>,
}

// NOTE: [`Drop`] should be implemented for [`State`] and *not* the [`DedicatedExecutor`] as the
// the executor can be cloned, whereas there will only be one instance of [`State`] for all
// executor clones.
impl Drop for State {
    fn drop(&mut self) {
        if self.tokio_rt_handle.is_some() {
            warn!("a `DedicatedExecutor` was dropped without calling `shutdown()`");
            self.tokio_rt_handle.take();
            self.shutdown_token.cancel();
        }

        // NOTE: ensure the thread is *not* panicking before polling the shared future
        //
        // See: https://github.com/rust-lang/futures-rs/issues/2575
        if !thread::panicking() && self.shutdown_completed.clone().now_or_never().is_none() {
            warn!("a `DedicatedExecutor` was dropped without waiting for worker termination");
        }

        // Join the thread but we don't about the result
        self.managing_tokio_rt_thread
            .take()
            .map(|thread| thread.join().ok());
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::unwrap_used)]
mod tests {
    use core::panic;
    use std::{
        panic::panic_any,
        sync::Barrier,
    };

    use tokio::sync::Barrier as AsyncBarrier;

    use super::*;

    const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

    fn exec() -> DedicatedExecutor {
        exec_with_threads(1)
    }

    fn exec2() -> DedicatedExecutor {
        exec_with_threads(2)
    }

    fn exec_with_threads(threads: usize) -> DedicatedExecutor {
        let mut tokio_rt_builder = runtime::Builder::new_multi_thread();
        tokio_rt_builder.worker_threads(threads);
        tokio_rt_builder.enable_all();

        DedicatedExecutor::new(
            "Test DedicatedExecutor",
            tokio_rt_builder,
            None,
            RUNTIME_SHUTDOWN_TIMEOUT,
        )
        .expect("failed to initialize runtime")
    }

    // Wait for the barrier and then return the `result` value
    async fn do_work(result: usize, barrier: Arc<Barrier>) -> usize {
        barrier.wait();
        result
    }

    // Wait for the barrier and then return the `result` value
    async fn do_work_async(result: usize, barrier: Arc<AsyncBarrier>) -> usize {
        barrier.wait().await;
        result
    }

    async fn test_io_runtime_multi_thread_impl(executor: DedicatedExecutor) {
        let io_rt_thread_id = std::thread::current().id();

        executor
            .spawn(async move {
                let rt_thread_id = std::thread::current().id();
                let spawned_thread_id =
                    parent::spawn_on_parent(async move { std::thread::current().id() }).await;

                assert_ne!(rt_thread_id, spawned_thread_id);
                assert_eq!(io_rt_thread_id, spawned_thread_id);
            })
            .await
            .expect("task errored");
    }

    #[tokio::test]
    async fn basic() {
        let barrier = Arc::new(Barrier::new(2));
        let executor = exec();

        let executor_task = executor.spawn(do_work(42, Arc::clone(&barrier)));

        // NOTE: the `executor_task` will never complete if it runs on the main Tokio thread (as
        // this test is not using the multi-threaded version of the runtime and the call
        // `barrier.wait()` blocks the Tokio thread)
        barrier.wait();

        assert_eq!(executor_task.await.expect("task errored"), 42);

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn basic_clone() {
        let barrier = Arc::new(Barrier::new(2));
        let executor = exec();

        // Running task on a clone of the executor should work as normal
        let executor_task = executor.clone().spawn(do_work(42, Arc::clone(&barrier)));
        barrier.wait();

        assert_eq!(executor_task.await.expect("task errored"), 42);

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn drop_empty_executor() {
        // Drop should not panic or fail on an exector not doing anything (i.e. "empty")
        exec();
    }

    #[tokio::test]
    async fn drop_clone() {
        let barrier = Arc::new(Barrier::new(2));
        let executor = exec();

        // Clones should drop cleanly without dropping the executor
        drop(executor.clone());

        let executor_task = executor.clone().spawn(do_work(42, Arc::clone(&barrier)));
        barrier.wait();
        assert_eq!(executor_task.await.expect("task errored"), 42);

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    #[should_panic(expected = "foo")]
    async fn just_panic() {
        struct Foobar(DedicatedExecutor);

        impl Drop for Foobar {
            fn drop(&mut self) {
                self.0.join().now_or_never();
            }
        }

        let executor = exec();
        let _foo = Foobar(executor);

        // This must not lead to a double-panic and `SIGILL`
        //
        // See: https://www.gnu.org/software/libc/manual/html_node/Program-Error-Signals.html
        panic!("foo");
    }

    #[tokio::test]
    async fn multi_task() {
        let barrier = Arc::new(Barrier::new(3));

        // Create a runtime with 2 threads
        let executor = exec2();
        let executor_task1 = executor.spawn(do_work(21, Arc::clone(&barrier)));
        let executor_task2 = executor.spawn(do_work(42, Arc::clone(&barrier)));

        // Block main thread until completion of other 2 tasks
        barrier.wait();

        assert_eq!(executor_task1.await.expect("task errored"), 21);
        assert_eq!(executor_task2.await.expect("task errored"), 42);

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn tokio_spawn() {
        let executor = exec2();

        // Spawn a task that spawns another task to ensure that they both run on the compute
        // runtime
        let executor_task = executor.spawn(async move {
            // Spawn a seperate task
            let other_task = tokio::task::spawn(async { 25usize });
            other_task.await.expect("join errored")
        });

        // Validate that the inner task ran to completion and it did not panic
        assert_eq!(executor_task.await.expect("task errored"), 25);

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn panic_on_runtime_str() {
        let executor = exec();

        let executor_task = executor.spawn(async move {
            if true {
                panic!("oh noes!");
            } else {
                42
            }
        });

        match executor_task.await.unwrap_err() {
            DedicatedExecutorError::TaskPanicked(msg) => assert_eq!("oh noes!", msg),
            DedicatedExecutorError::WorkerGone => panic!("unexpected error"),
        }
    }

    #[tokio::test]
    async fn panic_on_runtime_string() {
        let executor = exec();

        let executor_task = executor.spawn(async move {
            if true {
                panic!("{}, {}", 1, 2);
            } else {
                42
            }
        });

        match executor_task.await.unwrap_err() {
            DedicatedExecutorError::TaskPanicked(msg) => assert_eq!("1, 2", msg),
            DedicatedExecutorError::WorkerGone => panic!("unexpected error"),
        }
    }

    #[tokio::test]
    async fn panic_on_runtime_other() {
        let executor = exec();

        let executor_task = executor.spawn(async move {
            if true {
                panic_any(1);
            } else {
                42
            }
        });

        match executor_task.await.unwrap_err() {
            DedicatedExecutorError::TaskPanicked(msg) => assert_eq!("unknown internal error", msg),
            DedicatedExecutorError::WorkerGone => panic!("unexpected error"),
        }
    }

    #[tokio::test]
    async fn executor_shutdown_while_running_task() {
        let barrier1 = Arc::new(Barrier::new(2));
        let captured1 = Arc::clone(&barrier1);
        let barrier2 = Arc::new(Barrier::new(2));
        let captured2 = Arc::clone(&barrier2);

        let executor = exec();
        let executor_task = executor.spawn(async move {
            captured1.wait();
            do_work(42, captured2).await
        });
        barrier1.wait();

        executor.shutdown();
        // Block main thread until completion of the outstanding task
        barrier2.wait();

        assert_eq!(executor_task.await.expect("task errored"), 42);

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn executor_submit_task_after_clone_shutdown() {
        let executor = exec();

        // Shut down the clone, but not the `exec`
        executor.clone().join().await.expect("join errored");

        // Simulate trying to submit a task once runtime has shutdown
        let executor_task = executor.spawn(async { 11 });

        assert!(matches!(
            executor_task.await.unwrap_err(),
            DedicatedExecutorError::WorkerGone
        ));

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn executor_join() {
        let executor = exec();
        // Ensure join doesn't hang
        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn executor_join2() {
        let executor = exec();
        // Ensure join doesn't hang
        executor.join().await.expect("join errored");
        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn executor_clone_join() {
        let executor = exec();
        // Ensure join doesn't hang
        executor.clone().join().await.expect("join errored");
        executor.clone().join().await.expect("join errored");
        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn drop_receiver() {
        // Create an empty executor
        let executor = exec();

        // Create first blocked task
        let barrier1_pre = Arc::new(AsyncBarrier::new(2));
        let barrier1_pre_captured = Arc::clone(&barrier1_pre);
        let barrier1_post = Arc::new(AsyncBarrier::new(2));
        let barrier1_post_captured = Arc::clone(&barrier1_post);

        let executor_task1 = executor.spawn(async move {
            barrier1_pre_captured.wait().await;
            do_work_async(11, barrier1_post_captured).await
        });
        barrier1_pre.wait().await;

        // Create first blocked task
        let barrier2_pre = Arc::new(AsyncBarrier::new(2));
        let barrier2_pre_captured = Arc::clone(&barrier2_pre);
        let barrier2_post = Arc::new(AsyncBarrier::new(2));
        let barrier2_post_captured = Arc::clone(&barrier2_post);

        let executor_task2 = executor.spawn(async move {
            barrier2_pre_captured.wait().await;
            do_work_async(22, barrier2_post_captured).await
        });
        barrier2_pre.wait().await;

        // Cancel a task 1
        drop(executor_task1);

        // Wait on cancellation, evidient by `barrier2_post` Arc count going from 2 to 1 (this
        // might take a short while)
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if Arc::strong_count(&barrier1_post) == 1 {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await
            }
        })
        .await
        .expect("timeout reached");

        // Unblock task 2
        barrier2_post.wait().await;
        assert_eq!(executor_task2.await.expect("task errored"), 22);
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if Arc::strong_count(&barrier2_post) == 1 {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await
            }
        })
        .await
        .expect("timeout reached");

        executor.join().await.expect("join errored");
    }

    #[tokio::test]
    async fn io_runtime_multi_thread() {
        let mut tokio_rt_builder = runtime::Builder::new_multi_thread();
        tokio_rt_builder.worker_threads(1);

        let executor = DedicatedExecutor::new(
            "Test DedicatedExecutor",
            tokio_rt_builder,
            None,
            RUNTIME_SHUTDOWN_TIMEOUT,
        )
        .expect("failed to initialize runtime");

        test_io_runtime_multi_thread_impl(executor).await;
    }

    #[tokio::test]
    async fn io_runtime_current_thread() {
        let tokio_rt_builder = runtime::Builder::new_current_thread();

        let executor = DedicatedExecutor::new(
            "Test DedicatedExecutor",
            tokio_rt_builder,
            None,
            RUNTIME_SHUTDOWN_TIMEOUT,
        )
        .expect("failed to initialize runtime");

        test_io_runtime_multi_thread_impl(executor).await;
    }
}
