use darling::FromField;
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

#[derive(Debug, Default, FromField)]
#[darling(attributes(definition_checksum), default)]
struct FieldOptions {
    /// Marks a field as recursive, using field name + type string instead of recursive checksum
    /// This prevents infinite loops during static initialization while maintaining schema change detection
    #[darling(default)]
    recursive_definition: bool,
}

pub fn derive_definition_checksum(
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
        Data::Struct(struct_data) => derive_definition_checksum_struct(ident, struct_data, errors),
        Data::Enum(enum_data) => derive_definition_checksum_enum(ident, enum_data, errors),
        _ => bail!("DefinitionChecksum can only be derived for structs and enums"),
    }
}

fn derive_definition_checksum_struct(
    ident: syn::Ident,
    struct_data: &syn::DataStruct,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let mut definition_checksum_parts = Vec::new();

    // Add the struct name itself to the checksum
    let ident_string = ident.to_string();
    definition_checksum_parts.push(quote! {
        hasher.update(#ident_string.as_bytes());
    });

    for field in &struct_data.fields {
        let Some(field_ident) = &field.ident else {
            emit!(
                errors,
                syn::Error::new_spanned(field, "struct field must have an identifier")
            );
            continue;
        };

        let field_name = field_ident.to_string();
        let field_type = ty_to_string(&field.ty);

        // Parse field-level attributes
        let field_options = match FieldOptions::from_field(field) {
            Ok(options) => options,
            Err(err) => {
                emit!(errors, err);
                FieldOptions::default()
            }
        };

        // Critical recursive handling logic:
        if field_options.recursive_definition {
            // For recursive fields: use field name + type string to break recursion cycles
            // This prevents infinite loops during static initialization while still
            // detecting schema changes (e.g., changing Vec<PropSchemaV1> to HashMap<String, PropSchemaV1>)
            definition_checksum_parts.push(quote! {
                hasher.update(#field_name.as_bytes());
                hasher.update(#field_type.as_bytes());
            });
        } else {
            // For non-recursive fields: use field name + recursive DefinitionChecksum
            // This provides full schema change detection for nested types
            let field_ty = &field.ty;
            definition_checksum_parts.push(quote! {
                hasher.update(#field_name.as_bytes());
                hasher.update(<#field_ty as crate::definition_checksum::DefinitionChecksum>::definition_checksum().as_bytes());
            });
        }
    }

    errors.into_result()?;

    let (definition_checksum_static_ident, definition_checksum_body) =
        definition_checksum_static_ident_and_body(&ident, definition_checksum_parts);

    let output = quote! {
        impl crate::definition_checksum::DefinitionChecksum for #ident {
            fn definition_checksum() -> ::si_events::workspace_snapshot::Checksum {
                *#definition_checksum_static_ident
            }
        }

        static #definition_checksum_static_ident: ::std::sync::LazyLock<::si_events::workspace_snapshot::Checksum> =
            ::std::sync::LazyLock::new(|| {
                #definition_checksum_body
            });

        ::inventory::submit! {
            crate::definition_checksum::DefinitionChecksumInventoryItem::new(
                #ident_string,
                &#definition_checksum_static_ident,
            )
        };
    };

    Ok(output.into())
}

fn derive_definition_checksum_enum(
    ident: syn::Ident,
    enum_data: &syn::DataEnum,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let ident_string = ident.to_string();
    let mut definition_checksum_parts = vec![quote! {
        hasher.update(#ident_string.as_bytes());
    }];

    // For enums, we include all variant information in the definition checksum
    // This means adding/removing variants or changing variant fields will trigger rebuilds
    for variant in &enum_data.variants {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        definition_checksum_parts.push(quote! {
            hasher.update(#variant_name.as_bytes());
        });

        match &variant.fields {
            // Variant is a struct structure with named fields
            syn::Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    let Some(field_ident) = &field.ident else {
                        emit!(
                            errors,
                            syn::Error::new_spanned(field, "named field must have an identifier")
                        );
                        continue;
                    };

                    let field_name = field_ident.to_string();
                    let field_type = ty_to_string(&field.ty);

                    // Parse field-level attributes for enum fields too
                    let field_options = match FieldOptions::from_field(field) {
                        Ok(options) => options,
                        Err(err) => {
                            emit!(errors, err);
                            FieldOptions::default()
                        }
                    };

                    if field_options.recursive_definition {
                        definition_checksum_parts.push(quote! {
                            hasher.update(#field_name.as_bytes());
                            hasher.update(#field_type.as_bytes());
                        });
                    } else {
                        let field_ty = &field.ty;
                        definition_checksum_parts.push(quote! {
                            hasher.update(#field_name.as_bytes());
                            hasher.update(<#field_ty as crate::definition_checksum::DefinitionChecksum>::definition_checksum().as_bytes());
                        });
                    }
                }
            }
            // Variant is a tuple structure with unnamed fields
            syn::Fields::Unnamed(fields_unnamed) => {
                for (index, field) in fields_unnamed.unnamed.iter().enumerate() {
                    let field_name = format!("field_{index}");
                    let field_type = ty_to_string(&field.ty);

                    // Parse field-level attributes for tuple fields
                    let field_options = match FieldOptions::from_field(field) {
                        Ok(options) => options,
                        Err(err) => {
                            emit!(errors, err);
                            FieldOptions::default()
                        }
                    };

                    if field_options.recursive_definition {
                        definition_checksum_parts.push(quote! {
                            hasher.update(#field_name.as_bytes());
                            hasher.update(#field_type.as_bytes());
                        });
                    } else {
                        let field_ty = &field.ty;
                        definition_checksum_parts.push(quote! {
                            hasher.update(#field_name.as_bytes());
                            hasher.update(<#field_ty as crate::definition_checksum::DefinitionChecksum>::definition_checksum().as_bytes());
                        });
                    }
                }
            }
            // Variant has no fields (unit variant)
            syn::Fields::Unit => {
                // No additional checksum data needed beyond the variant name
            }
        }
    }

    errors.into_result()?;

    let (definition_checksum_static_ident, definition_checksum_body) =
        definition_checksum_static_ident_and_body(&ident, definition_checksum_parts);

    let output = quote! {
        impl crate::definition_checksum::DefinitionChecksum for #ident {
            fn definition_checksum() -> ::si_events::workspace_snapshot::Checksum {
                *#definition_checksum_static_ident
            }
        }

        static #definition_checksum_static_ident: ::std::sync::LazyLock<::si_events::workspace_snapshot::Checksum> =
            ::std::sync::LazyLock::new(|| {
                #definition_checksum_body
            });

        ::inventory::submit! {
            crate::definition_checksum::DefinitionChecksumInventoryItem::new(
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
    let definition_checksum_static_ident =
        format_ident!("{}_DEFINITION_CHECKSUM", ident.to_string().to_uppercase());

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
