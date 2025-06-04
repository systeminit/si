use manyhow::emit;
use quote::quote;
use syn::{
    Data,
    DeriveInput,
};

pub fn derive_frontend_object(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        ..
    } = input.clone();

    if !matches!(type_data, Data::Struct(_)) {
        emit!(
            errors,
            input,
            "FrontendObject can only be derived for structs"
        );
    }
    errors.into_result()?;

    let output = quote! {
        impl ::std::convert::TryFrom<#ident> for crate::object::FrontendObject {
            type Error = ::serde_json::Error;

            fn try_from(value: #ident) -> ::std::result::Result<Self, Self::Error> {
                let kind = crate::reference::ReferenceKind::#ident.to_string();
                let id = value.id.to_string();
                let checksum = crate::checksum::FrontendChecksum::checksum(&value).to_string();
                let data = ::serde_json::to_value(value)?;

                Ok(crate::object::FrontendObject {
                    kind,
                    id,
                    checksum,
                    data,
                })
            }
        }
    };

    Ok(output.into())
}
