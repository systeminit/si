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
mod progress;
mod qualification_check;
mod readiness;
mod resolver_function;
mod resource_sync;

pub use liveness::{LivenessStatus, LivenessStatusParseError};
pub use progress::{
    FunctionResult, FunctionResultFailure, FunctionResultFailureError, Message, OutputStream,
    ProgressMessage,
};
pub use qualification_check::{
    QualificationCheckComponent, QualificationCheckRequest, QualificationCheckResultSuccess,
};
pub use readiness::{ReadinessStatus, ReadinessStatusParseError};
pub use resolver_function::{ResolverFunctionRequest, ResolverFunctionResultSuccess};
pub use resource_sync::{ResourceSyncComponent, ResourceSyncRequest, ResourceSyncResultSuccess};

#[cfg(feature = "process")]
pub mod process;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::{Config, ConfigError, IncomingStream, Server};

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub use client::{Client, ClientError, CycloneClient, HttpClient, UdsClient};
