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

pub use cyclone::resolver_function::*;

#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub use server::{Config, ConfigBuilder, ConfigError, CycloneStream, Server};

#[cfg(feature = "client")]
pub mod client;
