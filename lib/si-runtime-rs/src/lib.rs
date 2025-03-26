//! Common Tokio runtime related behavior.

use std::{
    io,
    ops::Deref,
    str::FromStr,
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

const MAX_WORKER_THREADS: usize = 32;

pub use core_affinity::CoreId;
pub use tokio_dedicated_executor::{
    DedicatedExecutor, DedicatedExecutorError, DedicatedExecutorInitializeError,
    DedicatedExecutorJoinError,
};

/// Builds a main/primary Tokio [`Runtime`] with sensible defaults.
pub fn main_tokio_runtime(runtime_name: impl Into<String>) -> io::Result<Runtime> {
    common_tokio_builder("main", runtime_name)
        .thread_stack_size(DEFAULT_TOKIO_RT_THREAD_STACK_SIZE)
        .max_blocking_threads(DEFAULT_TOKIO_RT_BLOCKING_POOL_SIZE)
        // Enables using net, process, signal, and some I/O types
        .enable_io()
        .build()
}

/// Builds a main/primary Tokio [`Runtime`] with worker threads pinned to CPU cores.
///
/// # References
///
/// The implementation for CPU pinning is adapted from the blog post "How to configure CPU cores to
/// be used in a Tokio application with core_affinity" by Christian Visintin.
///
/// See: <https://blog.veeso.dev/blog/en/how-to-configure-cpu-cores-to-be-used-on-a-tokio-with-core--affinity/>
pub fn main_tokio_runtime_with_core_affinitiy(
    runtime_name: impl Into<String>,
    cpu_cores: Vec<CoreId>,
) -> io::Result<Runtime> {
    if cpu_cores.is_empty() {
        return Err(io::Error::other("cpu_cores cannot be an empty vec"));
    }

    common_main_tokio_builder("main", runtime_name)
        .worker_threads(cpu_cores.len().max(MAX_WORKER_THREADS))
        // After each thread is started but before it starts doing work.
        //
        // Select a random core from the set of `CoreId`s and assign the thread via the
        // `core_affinity` crate.
        .on_thread_start(move || {
            use rand::seq::SliceRandom;

            let core = {
                let mut rng = rand::thread_rng();
                *cpu_cores
                    .choose(&mut rng)
                    .expect("cpu_cores is non-empty so will always return an entry")
            };

            if !core_affinity::set_for_current(core) {
                // The setup of the main Tokio runtime happens *before* the configuration and setup
                // of tracing/telemetry so a `warn!()`/`error!()` won't be reported.
                //
                // Let's hope this isn't too distasteful...
                eprintln!(
                    "si-runtime: failed to pin tokio worker/blocking thread to core {}",
                    core.id
                );
            }
        })
        .build()
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

#[derive(Clone, Debug)]
pub struct CoreIds(Vec<CoreId>);

impl CoreIds {
    pub fn into_inner(self) -> Vec<CoreId> {
        self.0
    }

    pub fn to_vec(&self) -> Vec<CoreId> {
        self.0.clone()
    }
}

impl FromStr for CoreIds {
    type Err = io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ids = get_cpu_cores_from_range_expr(s)?;
        Ok(Self(ids))
    }
}

impl Deref for CoreIds {
    type Target = [CoreId];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

/// Returns a list of filtered [`CoreId`]s corresponding to CPU cores.
pub fn get_cpu_cores(selection: impl Into<Option<Vec<usize>>>) -> io::Result<Vec<CoreId>> {
    let available_cores =
        core_affinity::get_core_ids().ok_or(io::Error::other("failed to get available cores"))?;

    match selection.into() {
        // Filter down the full list of available_cores to only those cores specified in `selection`
        Some(selection) => {
            let cores = available_cores
                .into_iter()
                .filter(|core| selection.contains(&core.id))
                .collect::<Vec<_>>();

            Ok(cores)
        }
        // No filter provided, return full list of available cores
        None => Ok(available_cores),
    }
}

/// Returns a list of filtered [`CoreId`]s from a range expression.
///
/// # Format
///
/// The `range_expr` can contain:
///
/// - A single number, ex: `"2"` -> `[2]`
/// - A list of numbers, seperated with commas, ex: `"0,1,45"` -> `[0,1,4,5]`
/// - A range of numbers, expressed as 2 numbers with a dash, ex: `"0-3"` -> `[0,1,2,3]`
/// - A combination of numbers and ranges, seperated with commas, ex: `"8,0-3,12"` ->
///   `[0,1,2,3,8,12]`
#[inline]
pub fn get_cpu_cores_from_range_expr(range_expr: &str) -> io::Result<Vec<CoreId>> {
    let selection = parse_range_expr(range_expr)?;
    get_cpu_cores(selection)
}

#[inline]
fn compute_tokio_builder(runtime_name: impl Into<String>) -> Builder {
    // NOTE: importantly this runtime does not have `enable_io()` turned on
    common_tokio_builder("compute", runtime_name)
}

#[inline]
fn common_main_tokio_builder(category: &'static str, runtime_name: impl Into<String>) -> Builder {
    let mut builder = common_tokio_builder(category, runtime_name);
    builder
        .thread_stack_size(DEFAULT_TOKIO_RT_THREAD_STACK_SIZE)
        .max_blocking_threads(DEFAULT_TOKIO_RT_BLOCKING_POOL_SIZE)
        // Enables using net, process, signal, and some I/O types
        .enable_io();
    builder
}

#[inline]
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

fn parse_range_expr(expr: &str) -> io::Result<Vec<usize>> {
    let mut ids = Vec::new();

    if expr.is_empty() {
        return Ok(ids);
    }

    for element in expr.split(',') {
        if element.contains('-') {
            let mut spliterator = element.splitn(2, '-');
            match (spliterator.next(), spliterator.next(), spliterator.next()) {
                (Some(start_str), Some(end_str), None) => {
                    let start = start_str.parse::<usize>().map_err(io::Error::other)?;
                    let end = end_str.parse::<usize>().map_err(io::Error::other)?;
                    ids.extend(start..=end);
                }
                _ => return Err(io::Error::other("failed to parse '<start>-<end>' range")),
            }
        } else {
            let id = element.parse::<usize>().map_err(io::Error::other)?;
            ids.push(id);
        }
    }

    ids.sort();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_range_expr {
        use super::*;

        #[test]
        fn empty() {
            let range = parse_range_expr("").expect("failed to parse range expr");
            assert_eq!(range, vec![])
        }

        #[test]
        fn with_single_number() {
            let range = parse_range_expr("2").expect("failed to parse range expr");
            assert_eq!(range, vec![2])
        }

        #[test]
        fn with_multiple_numbers() {
            let range = parse_range_expr("4,2").expect("failed to parse range expr");
            assert_eq!(range, vec![2, 4])
        }

        #[test]
        fn with_range() {
            let range = parse_range_expr("0-3").expect("failed to parse range expr");
            assert_eq!(range, vec![0, 1, 2, 3])
        }

        #[test]
        fn with_numbers_and_ranges() {
            let range = parse_range_expr("2,4-7,12").expect("failed to parse range expr");
            assert_eq!(range, vec![2, 4, 5, 6, 7, 12])
        }
    }
}
