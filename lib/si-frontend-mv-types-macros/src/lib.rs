use manyhow::{
    bail,
    emit,
    manyhow,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data,
    DeriveInput,
};

mod materialized_view;

use crate::materialized_view::derive_materialized_view;

#[manyhow]
#[proc_macro_derive(FrontendChecksum, attributes(frontend_checksum))]
pub fn frontend_checksum_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_frontend_checksum(input, errors)
}

fn derive_frontend_checksum(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        ..
    } = input.clone();

    match &type_data {
        Data::Struct(struct_data) => derive_frontend_checksum_struct(ident, struct_data, errors),
        Data::Enum(enum_data) => derive_frontend_checksum_enum(ident, enum_data, errors),
        _ => bail!("FrontendChecksum can only be derived for structs and enums"),
    }
}

fn derive_frontend_checksum_struct(
    ident: syn::Ident,
    struct_data: &syn::DataStruct,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let mut field_update_parts = Vec::new();
    for field in &struct_data.fields {
        let Some(field_ident) = &field.ident else {
            emit!(
                errors,
                syn::Error::new_spanned(field, "struct field must have an identifier")
            );
            continue;
        };
        let field_name = field_ident.to_string();
        field_update_parts.push(quote! {
            hasher.update(#field_name.as_bytes());
            hasher.update(
                crate::checksum::FrontendChecksum::checksum(&self.#field_ident).as_bytes()
            );
        })
    }
    errors.into_result()?;

    let mut field_updates = TokenStream::new();
    field_updates.extend(field_update_parts);

    let checksum_fn = quote! {
        fn checksum(&self) -> ::si_events::workspace_snapshot::Checksum {
            let mut hasher = ::si_events::workspace_snapshot::ChecksumHasher::new();
            #field_updates
            hasher.finalize()
        }
    };

    let output = quote! {
        impl crate::checksum::FrontendChecksum for #ident {
            #checksum_fn
        }
    };

    Ok(output.into())
}

fn derive_frontend_checksum_enum(
    ident: syn::Ident,
    enum_data: &syn::DataEnum,
    _errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let mut variant_match_arms = Vec::new();

    // Iterate through each defined variant for the enum
    for variant in &enum_data.variants {
        let variant_ident = &variant.ident;

        let token_stream = match &variant.fields {
            // Variant is a struct structure with fields
            syn::Fields::Named(fields_named) => {
                let fields_named = syn::punctuated::Punctuated::<_, syn::Token![,]>::from_iter(
                    fields_named
                        .named
                        .iter()
                        .filter_map(|field| field.ident.clone()),
                );
                // Checksum each field
                let checksum_fields = fields_named.iter().map(|field_ident| {
                    let field_name = field_ident.to_string();
                    quote! {
                        hasher.update(#field_name.as_bytes());
                        hasher.update(
                            crate::checksum::FrontendChecksum::checksum(#field_ident).as_bytes()
                        );
                    }
                });
                let checksum_fields_stream = TokenStream::from_iter(checksum_fields);

                let variant_name = variant_ident.to_string();
                quote! {
                    #ident::#variant_ident { #fields_named } => {
                        hasher.update(#variant_name.as_bytes());
                        #checksum_fields_stream
                    }
                }
            }
            // Variant is a tuple structure with unnamed fields
            syn::Fields::Unnamed(fields_unnamed) => {
                let fields_count = fields_unnamed.unnamed.len();
                let fields: Vec<_> = (0..fields_count)
                    .map(|num| {
                        syn::Ident::new(&format!("field_{num}"), proc_macro2::Span::call_site())
                    })
                    .collect();
                let fields_named =
                    syn::punctuated::Punctuated::<_, syn::Token![,]>::from_iter(fields.iter());
                // Checksum each field
                let checksum_fields = fields.iter().map(|field_ident| {
                    let field_name = field_ident.to_string();
                    quote! {
                        hasher.update(#field_name.as_bytes());
                        hasher.update(
                            crate::checksum::FrontendChecksum::checksum(#field_ident).as_bytes()
                        );
                    }
                });
                let checksum_fields_stream = TokenStream::from_iter(checksum_fields);

                let variant_name = variant_ident.to_string();
                quote! {
                    #ident::#variant_ident(#fields_named) => {
                        hasher.update(#variant_name.as_bytes());
                        #checksum_fields_stream
                    }
                }
            }
            // Variant has no fields and is a "unit" structure
            syn::Fields::Unit => {
                // Use the `ToString` trait as the impl
                quote! {
                    #ident::#variant_ident => {
                        hasher.update(self.to_string().as_bytes());
                    }
                }
            }
        };

        variant_match_arms.push(token_stream);
    }

    let mut match_arms_stream = TokenStream::new();
    match_arms_stream.extend(variant_match_arms);

    let checksum_fn = quote! {
        fn checksum(&self) -> ::si_events::workspace_snapshot::Checksum {
            let mut hasher = ::si_events::workspace_snapshot::ChecksumHasher::new();

            match self {
                #match_arms_stream
            }

            hasher.finalize()
        }
    };

    let output = quote! {
        impl crate::checksum::FrontendChecksum for #ident {
            #checksum_fn
        }
    };

    Ok(output.into())
}

#[manyhow]
#[proc_macro_derive(FrontendObject, attributes(frontend_object))]
pub fn frontend_object_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_frontend_object(input, errors)
}

fn derive_frontend_object(
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

#[manyhow]
#[proc_macro_derive(Refer, attributes(refer))]
pub fn refer_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_refer(input, errors)
}

fn derive_refer(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        ..
    } = input.clone();

    let Data::Struct(struct_data) = type_data else {
        bail!("Refer can only be derived for structs");
    };

    let mut id_type = None;
    let mut id_field = None;
    for field in &struct_data.fields {
        let Some(field_ident) = &field.ident else {
            continue;
        };
        let field_ty = &field.ty;

        if field_ident == "id" {
            id_type = Some(field_ty.clone());
            id_field = Some(field_ident.clone());
        }
    }
    errors.into_result()?;

    let Some(id_field) = id_field else {
        bail!(input, "'id' field must be present");
    };
    let Some(id_type) = id_type else {
        bail!(input, "'id' field must have a type");
    };

    let refer_impl = quote! {
        impl crate::reference::Refer<#id_type> for #ident {
            fn reference_kind(&self) -> crate::reference::ReferenceKind {
                self.into()
            }

            fn reference_id(&self) -> crate::reference::ReferenceId<#id_type> {
                crate::reference::ReferenceId(self.#id_field)
            }
        }
    };

    let from_for_reference_impl = quote! {
        impl From<&#ident> for crate::reference::Reference<#id_type> {
            fn from(value: &#ident) -> Self {
                crate::reference::Refer::reference(value)
            }
        }
    };

    let from_for_reference_kind_impl = quote! {
        impl From<&#ident> for crate::reference::ReferenceKind {
            fn from(value: &#ident) -> Self {
                crate::reference::ReferenceKind::#ident
            }
        }
    };

    let output = quote! {
        #refer_impl
        #from_for_reference_impl
        #from_for_reference_kind_impl
    };

    Ok(output.into())
}

#[manyhow]
#[proc_macro_derive(MV, attributes(mv))]
pub fn materialized_view_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_materialized_view(input, errors)
}
