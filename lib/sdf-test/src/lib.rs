//! This crate provides functionality for only SDF-specific tests.

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

use std::env;

use rand::{
    Rng,
    distributions::Alphanumeric,
    thread_rng,
};
use si_data_spicedb::SpiceDbConfig;

const ENV_VAR_SPICEDB_URL: &str = "SI_TEST_SPICEDB_URL";

pub mod helpers;

/// Provides a [`SpiceDbConfig`] for SDF tests.
///
/// # Panics
///
/// This function will panic is the URL passed in via an environment variable cannot be parsed.
pub fn spicedb_config() -> SpiceDbConfig {
    let mut config = SpiceDbConfig::default();
    #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
    if let Ok(value) = env::var(ENV_VAR_SPICEDB_URL) {
        config.endpoint = value.parse().expect("failed to parse spicedb url");
    }

    let mut rng = thread_rng();
    let random_string: String = (0..12).map(|_| rng.sample(Alphanumeric) as char).collect();
    config.preshared_key = random_string.into();
    config
}
