#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

pub use tracing;

#[cfg(feature = "application")]
mod application;
#[cfg(feature = "application")]
pub use application::{
    init, start_tracing_level_signal_handler_task, Config, ConfigBuilderError, Error as InitError,
};

#[cfg(feature = "library")]
mod library;
#[cfg(feature = "library")]
pub use library::{
    prelude, Client, ClientError, NoopClient, TelemetryClient, TelemetryLevel, TracingLevel,
    UpdateOpenTelemetry, Verbosity,
};
