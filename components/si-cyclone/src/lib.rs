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

mod liveness;
mod readiness;
pub mod resolver_function;

pub(crate) use liveness::{LivenessStatus, LivenessStatusParseError};
pub(crate) use readiness::{ReadinessStatus, ReadinessStatusParseError};

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::{telemetry, Config, ConfigError, IncomingStream, Server};

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{
    Client, ClientError, Connection, CycloneClient, HttpClient, ResolverFunctionExecutionError,
    UdsClient,
};
