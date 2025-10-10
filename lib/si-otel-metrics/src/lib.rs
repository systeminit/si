//! Process metrics collection for System Initiative services using OpenTelemetry.
//!
//! This crate provides a simple API for instrumenting Rust services with process-level
//! metrics (memory, CPU) that are exported via OpenTelemetry.

#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    missing_docs
)]
#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use opentelemetry::metrics::{
    Meter,
    ObservableGauge,
};
use sysinfo::{
    Pid,
    ProcessRefreshKind,
    ProcessesToUpdate,
    System,
};
use thiserror::Error;

/// Errors that can occur during metrics initialization or collection.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to get current process information
    #[error("failed to get current process information")]
    ProcessInfoUnavailable,
}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Process metrics observer that registers observable gauges for memory and CPU metrics.
///
/// This struct maintains a reference to the system information and registers callbacks
/// with OpenTelemetry to report process metrics.
pub struct ProcessMetricsObserver {
    _memory_rss_gauge: ObservableGauge<u64>,
    _memory_virtual_gauge: ObservableGauge<u64>,
    _cpu_usage_gauge: ObservableGauge<f64>,
    _cpu_time_gauge: ObservableGauge<u64>,
}

impl ProcessMetricsObserver {
    /// Initialize process metrics collection for the current process.
    ///
    /// This function registers observable gauges that will automatically report
    /// process memory and CPU metrics to OpenTelemetry on each collection interval.
    ///
    /// # Arguments
    ///
    /// * `meter` - The OpenTelemetry meter to use for creating metric instruments
    ///
    /// # Returns
    ///
    /// Returns a `ProcessMetricsObserver` that must be kept alive for the duration
    /// of the metrics collection. Dropping this struct will stop metrics collection.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opentelemetry::global;
    /// use si_otel_metrics::ProcessMetricsObserver;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let meter = global::meter("my-service");
    ///     let _observer = ProcessMetricsObserver::init(meter).unwrap();
    ///
    ///     // Metrics will be collected automatically
    ///     // Keep _observer alive for the duration of the program
    /// }
    /// ```
    pub fn init(meter: Meter) -> Result<Self> {
        let pid = sysinfo::get_current_pid().map_err(|_| Error::ProcessInfoUnavailable)?;

        // Create a shared System instance wrapped in Arc for thread-safe access
        let system = Arc::new(parking_lot::Mutex::new(System::new_all()));

        // Perform initial CPU refresh to establish baseline for accurate cpu_usage() readings
        // Per sysinfo docs: "To start to have accurate CPU usage, a process needs to be refreshed twice"
        {
            let mut sys = system.lock();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[pid]),
                false,
                ProcessRefreshKind::default().with_cpu().with_memory(),
            );
        }

        // Register memory RSS (Resident Set Size) gauge
        let memory_rss_gauge = {
            let system = Arc::clone(&system);
            meter
                .u64_observable_gauge("process.runtime.memory.rss")
                .with_description("Process resident set size (physical memory)")
                .with_unit("bytes")
                .with_callback(move |observer| {
                    if let Some(memory_bytes) = get_process_memory_rss(&system, pid) {
                        observer.observe(memory_bytes, &[]);
                    }
                })
                .init()
        };

        // Register memory virtual size gauge
        let memory_virtual_gauge = {
            let system = Arc::clone(&system);
            meter
                .u64_observable_gauge("process.runtime.memory.virtual")
                .with_description("Process virtual memory size")
                .with_unit("bytes")
                .with_callback(move |observer| {
                    if let Some(memory_bytes) = get_process_memory_virtual(&system, pid) {
                        observer.observe(memory_bytes, &[]);
                    }
                })
                .init()
        };

        // Register CPU usage gauge (percentage)
        let cpu_usage_gauge = {
            let system = Arc::clone(&system);
            meter
                .f64_observable_gauge("process.runtime.cpu.usage")
                .with_description("Process CPU usage percentage")
                .with_unit("percent")
                .with_callback(move |observer| {
                    if let Some(cpu_usage) = get_process_cpu_usage(&system, pid) {
                        observer.observe(cpu_usage, &[]);
                    }
                })
                .init()
        };

        // Register accumulated CPU time gauge (milliseconds)
        let cpu_time_gauge = {
            let system = Arc::clone(&system);
            meter
                .u64_observable_gauge("process.runtime.cpu.time")
                .with_description("Accumulated CPU time in milliseconds")
                .with_unit("ms")
                .with_callback(move |observer| {
                    if let Some(cpu_time) = get_process_cpu_time(&system, pid) {
                        observer.observe(cpu_time, &[]);
                    }
                })
                .init()
        };

        Ok(Self {
            _memory_rss_gauge: memory_rss_gauge,
            _memory_virtual_gauge: memory_virtual_gauge,
            _cpu_usage_gauge: cpu_usage_gauge,
            _cpu_time_gauge: cpu_time_gauge,
        })
    }
}

/// Get the RSS (physical memory) usage of a process.
fn get_process_memory_rss(system: &Arc<parking_lot::Mutex<System>>, pid: Pid) -> Option<u64> {
    let mut sys = system.lock();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::default().with_cpu().with_memory(),
    );
    sys.process(pid).map(|process| process.memory())
}

/// Get the virtual memory usage of a process.
fn get_process_memory_virtual(system: &Arc<parking_lot::Mutex<System>>, pid: Pid) -> Option<u64> {
    let mut sys = system.lock();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::default().with_cpu().with_memory(),
    );
    sys.process(pid).map(|process| process.virtual_memory())
}

/// Get the CPU usage percentage of a process.
fn get_process_cpu_usage(system: &Arc<parking_lot::Mutex<System>>, pid: Pid) -> Option<f64> {
    let mut sys = system.lock();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::default().with_cpu().with_memory(),
    );
    sys.process(pid).map(|process| process.cpu_usage() as f64)
}

/// Get the accumulated CPU time of a process in milliseconds.
fn get_process_cpu_time(system: &Arc<parking_lot::Mutex<System>>, pid: Pid) -> Option<u64> {
    let mut sys = system.lock();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::default().with_cpu().with_memory(),
    );
    sys.process(pid)
        .map(|process| process.accumulated_cpu_time())
}
