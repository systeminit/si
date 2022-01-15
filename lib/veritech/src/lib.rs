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

#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub use server::{
    Config, ConfigBuilder, ConfigError, ConfigFile, CycloneSpec, CycloneStream, Server,
    ServerError, StandardConfig, StandardConfigFile,
};

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub use client::{Client, ClientError, ClientResult};
#[cfg(feature = "client")]
pub use cyclone::resolver_function::{
    OutputStream, ResolverFunctionRequest, ResolverFunctionResult, ResolverFunctionResultFailure,
};

pub(crate) const FINAL_MESSAGE_HEADER_KEY: &str = "X-Final-Message";

pub(crate) fn reply_mailbox_for_output(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.output")
}

pub(crate) fn reply_mailbox_for_result(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.result")
}
