use manyhow::manyhow;
use quote::ToTokens as _;

mod definition_checksum;
mod frontend_checksum;
mod frontend_object;
mod materialized_view;
mod refer;

use crate::{
    definition_checksum::derive_definition_checksum,
    frontend_checksum::derive_frontend_checksum,
    frontend_object::derive_frontend_object,
    materialized_view::derive_materialized_view,
    refer::derive_refer,
};

#[manyhow]
#[proc_macro_derive(DefinitionChecksum, attributes(definition_checksum))]
pub fn definition_checksum_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_definition_checksum(input, errors)
}

#[manyhow]
#[proc_macro_derive(FrontendChecksum, attributes(frontend_checksum))]
pub fn frontend_checksum_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_frontend_checksum(input, errors)
}

#[manyhow]
#[proc_macro_derive(FrontendObject, attributes(frontend_object))]
pub fn frontend_object_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_frontend_object(input, errors)
}

#[manyhow]
#[proc_macro_derive(Refer, attributes(refer))]
pub fn refer_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_refer(input, errors)
}

#[manyhow]
#[proc_macro_derive(MV, attributes(mv))]
pub fn materialized_view_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_materialized_view(input, errors)
}

fn ty_to_string(ty: &syn::Type) -> String {
    let mut result = String::new();
    for token in ty.to_token_stream() {
        result.push_str(&token.to_string());
    }

    result
}
