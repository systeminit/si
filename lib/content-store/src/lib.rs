//! This crate provides the ability to interface with content stores of varying kinds as well as
//! the ability to generate hashes for hashable content blobs.

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
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

mod hash;
mod pair;
mod store;

pub use hash::ContentHash;
pub use store::local::LocalStore;
pub use store::pg::tools::PgStoreTools;
pub use store::pg::PgStore;
pub use store::Store;
pub use store::StoreError;
