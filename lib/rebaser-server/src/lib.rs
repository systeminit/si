//! This crate provides the rebaser [`Server`].

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

pub use config::detect_and_configure_development;
pub use config::Config;
pub use config::ConfigBuilder;
pub use config::ConfigError;
pub use config::ConfigFile;
pub use server::Server;
pub use si_settings::StandardConfig;
pub use si_settings::StandardConfigFile;

mod config;
mod server;
