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
