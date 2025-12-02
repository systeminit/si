//! Provides a [`Server`] for "forklifting" data into a data warehouse stream.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

mod config;
mod middleware;
mod server;
pub use config::{
    Config,
    ConfigError,
    ConfigFile,
    StandardConfigFile,
};
pub use server::Server;
pub use si_service_endpoints::{
    DefaultServiceEndpoints,
    ServiceEndpointsConfig,
    server::EndpointsServer,
};
pub use si_settings::{
    ConfigMap,
    ParameterProvider,
};
