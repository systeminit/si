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
mod code_generation;
mod component_view;
mod liveness;
mod progress;
mod qualification_check;
mod readiness;
mod resolver_function;
mod resource_sync;
mod sensitive_container;

pub use code_generation::{CodeGenerated, CodeGenerationRequest, CodeGenerationResultSuccess};
pub use component_view::{ComponentKind, ComponentView, SystemView};
pub use liveness::{LivenessStatus, LivenessStatusParseError};
pub use progress::{
    FunctionResult, FunctionResultFailure, FunctionResultFailureError, Message, OutputStream,
    ProgressMessage,
};
pub use qualification_check::{
    QualificationCheckComponent, QualificationCheckRequest, QualificationCheckResultSuccess,
    QualificationSubCheck, QualificationSubCheckStatus,
};
pub use readiness::{ReadinessStatus, ReadinessStatusParseError};
pub use resolver_function::{
    ResolverFunctionComponent, ResolverFunctionRequest, ResolverFunctionResultSuccess,
};
pub use resource_sync::{ResourceSyncRequest, ResourceSyncResultSuccess};
pub use sensitive_container::{MaybeSensitive, SensitiveContainer, SensitiveString};

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

use chrono::Utc;

pub fn timestamp() -> u64 {
    u64::try_from(std::cmp::max(Utc::now().timestamp(), 0)).expect("timestamp not be negative")
}
