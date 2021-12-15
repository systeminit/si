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

mod canonical_file;
mod server;
pub use server::{
    build_service, service, Config, ConfigError, ConfigFile, IncomingStream, JwtSecretKey,
    MigrationMode, Server, StandardConfig, StandardConfigFile,
};
