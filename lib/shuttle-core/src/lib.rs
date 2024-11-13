//! This crate provides information relevant for working with shuttle instances.

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

/// The header key used to indicate to a running shuttle instance that it has consumed everything
/// and can shut down. The header value and message body are ignored.
pub const FINAL_MESSAGE_HEADER_KEY: &str = "X-Final-Message";

/// The header key used to indicate that destination subject needs an appended suffix. The value
/// for the header should be the suffix itself.
pub const DESTINATION_SUBJECT_SUFFIX_HEADER_KEY: &str = "X-Destination-Subject-Suffix";
