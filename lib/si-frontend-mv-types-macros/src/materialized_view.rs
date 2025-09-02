use std::str::FromStr;

use darling::{
    FromAttributes,
    FromField,
};
use manyhow::{
    bail,
    emit,
};
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
};
use si_events::materialized_view::BuildPriority;
use syn::{
    Data,
    DeriveInput,
    Path,
};

use crate::ty_to_string;

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(mv))]
struct MaterializedViewOptions {
    trigger_entity: Option<Path>,
    reference_kind: Option<Path>,
    build_priority: Option<String>,
}

#[derive(Debug, Default, FromField)]
#[darling(attributes(definition_checksum), default)]
struct FieldOptions {
    /// Marks a field as recursive, using field name + type string instead of recursive checksum
    /// This prevents infinite loops during static initialization while maintaining schema change detection
    #[darling(default)]
    recursive_definition: bool,
}

pub fn derive_materialized_view(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        attrs,
        ..
    } = input.clone();
    let struct_options = MaterializedViewOptions::from_attributes(&attrs)?;

    let Data::Struct(struct_data) = &type_data else {
        bail!("MV can only be derived for structs");
    };

    let mut definition_checksum_updates: Vec<TokenStream> = Vec::new();
    for field in &struct_data.fields {
        let Some(field_ident) = &field.ident else {
            emit!(
                errors,
                field,
                "MaterializedView can only be derived for structs with named fields"
            );
            continue;
        };

        let field_name = field_ident.to_string();
        let field_type = ty_to_string(&field.ty);

        // Parse field-level attributes to check for recursive_definition
        let field_options = match FieldOptions::from_field(field) {
            Ok(options) => options,
            Err(err) => {
                emit!(errors, err);
                FieldOptions::default()
            }
        };

        // Critical implementation: NO fallbacks - enforce DefinitionChecksum trait
        if field_options.recursive_definition {
            // For recursive fields: use field name + type string to break recursion cycles
            // This prevents infinite loops during static initialization while still
            // detecting schema changes (e.g., changing Vec<PropSchemaV1> to HashMap<String, PropSchemaV1>)
            definition_checksum_updates.push(quote! {
                hasher.update(#field_name.as_bytes());
                hasher.update(#field_type.as_bytes());
            });
        } else {
            // For non-recursive fields: use field name + recursive DefinitionChecksum
            // This provides full schema change detection for nested types
            // CRITICAL: No fallback - missing DefinitionChecksum implementations cause compilation errors
            let field_ty = &field.ty;
            definition_checksum_updates.push(quote! {
                hasher.update(#field_name.as_bytes());
                hasher.update(<#field_ty as crate::definition_checksum::DefinitionChecksum>::definition_checksum().as_bytes());
            });
        }
    }
    errors.into_result()?;

    let Some(trigger_entity) = struct_options.trigger_entity else {
        bail!(input, "MV must have a trigger_entity attribute");
    };
    let Some(self_reference_kind) = struct_options.reference_kind else {
        bail!(input, "MV must have a reference_kind attribute");
    };
    let build_priority = {
        let priority = match struct_options.build_priority {
            Some(priority_string) => {
                if let Ok(priority) = BuildPriority::from_str(&priority_string) {
                    priority
                } else {
                    bail!(
                        input,
                        "Invalid build_priority; must be one of the ::si_events::materialized_view::BuildPriority variants."
                    );
                }
            }
            None => BuildPriority::default(),
        }
        .to_string();
        format_ident!("{}", priority)
    };

    let definition_checksum = {
        let mut hash_updates = TokenStream::new();
        for update in definition_checksum_updates {
            hash_updates.extend(update);
        }

        quote! {
            let mut hasher = ::si_events::workspace_snapshot::ChecksumHasher::new();
            #hash_updates

            hasher.finalize()
        }
    };
    let checksum_static_ident = format_ident!(
        "{}_MATERIALIZED_VIEW_DEFINITION_CHECKSUM",
        ident.to_string().to_uppercase()
    );

    let output = quote! {
        impl crate::MaterializedView for #ident {
            fn kind() -> crate::reference::ReferenceKind {
                #self_reference_kind
            }

            fn trigger_entity() -> ::si_events::workspace_snapshot::EntityKind {
                #trigger_entity
            }

            fn definition_checksum() -> ::si_events::workspace_snapshot::Checksum {
                *#checksum_static_ident
            }

            fn build_priority() -> ::si_events::materialized_view::BuildPriority {
                ::si_events::materialized_view::BuildPriority::#build_priority
            }
        }

        static #checksum_static_ident: ::std::sync::LazyLock<::si_events::workspace_snapshot::Checksum> =
            ::std::sync::LazyLock::new(|| {
                #definition_checksum
            });

        ::inventory::submit! {
            crate::materialized_view::MaterializedViewInventoryItem::new(
                #self_reference_kind,
                #trigger_entity,
                ::si_events::materialized_view::BuildPriority::#build_priority,
                &#checksum_static_ident,
            )
        };
    };

    Ok(output.into())
}
