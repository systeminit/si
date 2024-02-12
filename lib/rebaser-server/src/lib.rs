//! This crate provides the rebaser [`Server`].

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
    rust_2018_idioms,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

pub use config::detect_and_configure_development;
pub use config::Config;
pub use config::ConfigBuilder;
pub use config::ConfigError;
pub use config::ConfigFile;
pub use rebaser_core::RebaserMessagingConfig;
pub use server::Server;
pub use si_settings::StandardConfig;
pub use si_settings::StandardConfigFile;

mod config;
mod server;
