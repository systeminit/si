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

#[manyhow]
#[proc_macro_derive(AllVersions, attributes(acceptable))]
pub fn derive_all_versions(
    input: TokenStream,
    errors: &mut Emitter,
) -> manyhow::Result<TokenStream2> {
    crate::all_versions::expand(input, errors)
}

#[manyhow]
#[proc_macro_derive(Container, attributes(acceptable))]
pub fn derive_container(input: TokenStream, errors: &mut Emitter) -> manyhow::Result<TokenStream2> {
    crate::container::expand(input, errors)
}

#[manyhow]
#[proc_macro_derive(CurrentContainer, attributes(acceptable))]
pub fn derive_current_container(
    input: TokenStream,
    errors: &mut Emitter,
) -> manyhow::Result<TokenStream2> {
    crate::current_container::expand(input, errors)
}

#[manyhow]
#[proc_macro_derive(Versioned, attributes(acceptable))]
pub fn derive_versioned(input: TokenStream, errors: &mut Emitter) -> manyhow::Result<TokenStream2> {
    crate::versioned::expand(input, errors)
}
