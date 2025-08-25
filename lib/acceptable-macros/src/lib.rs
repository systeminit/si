//! Derive macros that help reduce the amount of boilerplate when crafting API types using the
//! `acceptable` crate.

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

#[allow(unused_extern_crates)]
extern crate proc_macro;

use manyhow::{
    Emitter,
    manyhow,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod all_versions;
mod container;
mod current_container;
mod versioned;

/// Derives the `AllVersions` trait, marking a type as a container over all possible versions of a
/// message type.
#[manyhow]
#[proc_macro_derive(AllVersions, attributes(acceptable))]
pub fn derive_all_versions(
    input: TokenStream,
    errors: &mut Emitter,
) -> manyhow::Result<TokenStream2> {
    crate::all_versions::expand(input, errors)
}

/// Dervies the `Container` trait on a type which encloses a current version of a message type.
#[manyhow]
#[proc_macro_derive(Container, attributes(acceptable))]
pub fn derive_container(input: TokenStream, errors: &mut Emitter) -> manyhow::Result<TokenStream2> {
    crate::container::expand(input, errors)
}

/// Derives a `Container` implemented type associated with the current `AllVersions`-implemented
/// type.
#[manyhow]
#[proc_macro_derive(CurrentContainer, attributes(acceptable))]
pub fn derive_current_container(
    input: TokenStream,
    errors: &mut Emitter,
) -> manyhow::Result<TokenStream2> {
    crate::current_container::expand(input, errors)
}

/// Derives the `Versioned` trait for a unique version of a message type.
#[manyhow]
#[proc_macro_derive(Versioned, attributes(acceptable))]
pub fn derive_versioned(input: TokenStream, errors: &mut Emitter) -> manyhow::Result<TokenStream2> {
    crate::versioned::expand(input, errors)
}
