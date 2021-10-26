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

pub mod canonical_command;
mod liveness;
pub mod process;
mod readiness;
pub mod resolver_function;

pub use liveness::{LivenessStatus, LivenessStatusParseError};
pub use readiness::{ReadinessStatus, ReadinessStatusParseError};

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::{Config, ConfigError, IncomingStream, Server};

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub use client::{Client, ClientError, CycloneClient, HttpClient, UdsClient};
