use manyhow::{
    bail,
    emit,
};
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
};
use syn::{
    Data,
    DeriveInput,
    Ident,
};

use crate::ty_to_string;

pub fn derive_frontend_checksum(
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
    let mut definition_checksum_parts = Vec::new();
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
        });

        let field_type = ty_to_string(&field.ty);
        definition_checksum_parts.push(quote! {
            hasher.update(#field_name.as_bytes());
            hasher.update(#field_type.as_bytes());
        });
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

    let (definition_checksum_static_ident, definition_checksum_body) =
        definition_checksum_static_ident_and_body(&ident, definition_checksum_parts);
    let ident_string = ident.to_string();

    let output = quote! {
        impl crate::checksum::FrontendChecksum for #ident {
            #checksum_fn
        }

        static #definition_checksum_static_ident: ::std::sync::LazyLock<::si_events::workspace_snapshot::Checksum> =
            ::std::sync::LazyLock::new(|| {
                #definition_checksum_body
            });

        ::inventory::submit! {
            crate::checksum::FrontendChecksumInventoryItem::new(
                #ident_string,
                &#definition_checksum_static_ident,
            )
        };
    };

    Ok(output.into())
}

fn derive_frontend_checksum_enum(
    ident: syn::Ident,
    enum_data: &syn::DataEnum,
    _errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let mut variant_match_arms = Vec::new();
    let ident_string = ident.to_string();
    let mut definition_checksum_parts = {
        vec![quote! {
            hasher.update(#ident_string.as_bytes());
        }]
    };

    struct Named {
        ident: syn::Ident,
        ty: syn::Type,
    }

    // Iterate through each defined variant for the enum
    for variant in &enum_data.variants {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        definition_checksum_parts.push(quote! {
            hasher.update(#variant_name.as_bytes());
        });

        let checksum_updates_token_stream = match &variant.fields {
            // Variant is a struct structure with fields
            syn::Fields::Named(fields_named) => {
                let fields: Vec<_> = fields_named
                    .named
                    .iter()
                    .filter_map(|field| {
                        field.ident.as_ref().map(|ident| Named {
                            ident: ident.clone(),
                            ty: field.ty.clone(),
                        })
                    })
                    .collect();
                let fields_named = syn::punctuated::Punctuated::<_, syn::Token![,]>::from_iter(
                    fields.iter().map(|field| field.ident.clone()),
                );
                // Checksum each field
                let checksum_fields = fields.iter().map(|field| {
                    let field_ident = &field.ident;
                    let field_name = field.ident.to_string();
                    let type_string = ty_to_string(&field.ty);
                    definition_checksum_parts.push(quote! {
                        hasher.update(#field_name.as_bytes());
                        hasher.update(#type_string.as_bytes());
                    });

                    quote! {
                        hasher.update(#field_name.as_bytes());
                        hasher.update(
                            crate::checksum::FrontendChecksum::checksum(#field_ident).as_bytes()
                        );
                    }
                });
                let checksum_fields_stream = TokenStream::from_iter(checksum_fields);

                quote! {
                    #ident::#variant_ident { #fields_named } => {
                        hasher.update(#variant_name.as_bytes());
                        #checksum_fields_stream
                    }
                }
            }
            // Variant is a tuple structure with unnamed fields
            syn::Fields::Unnamed(fields_unnamed) => {
                let fields: Vec<_> = fields_unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(num, field)| Named {
                        ident: format_ident!(
                            "field_{}",
                            num,
                            span = proc_macro2::Span::call_site()
                        ),
                        ty: field.ty.clone(),
                    })
                    .collect();
                let fields_named = syn::punctuated::Punctuated::<_, syn::Token![,]>::from_iter(
                    fields.iter().map(|field| field.ident.clone()),
                );
                // Checksum each field
                let checksum_fields = fields.iter().map(|field| {
                    let field_ident = &field.ident;
                    let field_name = field.ident.to_string();
                    let type_string = ty_to_string(&field.ty);
                    definition_checksum_parts.push(quote! {
                        hasher.update(#field_name.as_bytes());
                        hasher.update(#type_string.as_bytes());
                    });

                    quote! {
                        hasher.update(#field_name.as_bytes());
                        hasher.update(
                            crate::checksum::FrontendChecksum::checksum(#field_ident).as_bytes()
                        );
                    }
                });
                let checksum_fields_stream = TokenStream::from_iter(checksum_fields);

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

        variant_match_arms.push(checksum_updates_token_stream);
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

    let (definition_checksum_static_ident, definition_checksum_body) =
        definition_checksum_static_ident_and_body(&ident, definition_checksum_parts);

    let output = quote! {
        impl crate::checksum::FrontendChecksum for #ident {
            #checksum_fn
        }

        static #definition_checksum_static_ident: ::std::sync::LazyLock<::si_events::workspace_snapshot::Checksum> =
            ::std::sync::LazyLock::new(|| {
                #definition_checksum_body
            });

        ::inventory::submit! {
            crate::checksum::FrontendChecksumInventoryItem::new(
                #ident_string,
                &#definition_checksum_static_ident,
            )
        };
    };

    Ok(output.into())
}

fn definition_checksum_static_ident_and_body(
    ident: &syn::Ident,
    parts: Vec<TokenStream>,
) -> (Ident, TokenStream) {
    let definition_checksum_static_ident = format_ident!(
        "{}_FRONTEND_CHECKSUM_DEFINITION_CHECKSUM",
        ident.to_string().to_uppercase()
    );

    let mut definition_checksum_updates = TokenStream::new();
    for part in parts {
        definition_checksum_updates.extend(part);
    }

    let definition_checksum_body = quote! {
        let mut hasher = ::si_events::workspace_snapshot::ChecksumHasher::new();
        #definition_checksum_updates

        hasher.finalize()
    };

    (definition_checksum_static_ident, definition_checksum_body)
}
