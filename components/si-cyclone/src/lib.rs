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

#[macro_use]
mod cfg;
pub mod liveness;
pub mod readiness;
pub mod resolver_function;

pub use liveness::{LivenessStatus, LivenessStatusParseError};
pub use readiness::{ReadinessStatus, ReadinessStatusParseError};

cfg_feature! {
    #![feature = "server"]

    pub mod server;
    pub use server::{Server, telemetry};
}

cfg_feature! {
    #![feature = "client"]

    pub mod client;
    pub use client::Client;
}
