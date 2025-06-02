use std::collections::HashSet;

use darling::{
    FromAttributes,
    util::path_to_string,
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
}

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(mv))]
struct MaterializedViewReferenceOptions {
    reference_kind: Option<Path>,
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
    let mut reference_kinds: HashSet<Path> = HashSet::new();
    for field in &struct_data.fields {
        let field_attrs = MaterializedViewReferenceOptions::from_attributes(&field.attrs)?;
        if let Some(reference_kind) = &field_attrs.reference_kind {
            reference_kinds.insert(reference_kind.clone());
        }

        let maybe_checksum_field_name = field.ident.as_ref().map(|i| i.to_string());
        let checksum_field_type = ty_to_string(&field.ty);

        if let Some(checksum_field_name) = maybe_checksum_field_name {
            definition_checksum_updates.push(quote! {
                hasher.update(#checksum_field_name.as_bytes());
                hasher.update(#checksum_field_type.as_bytes());
            });
        } else {
            emit!(
                errors,
                field,
                "Unable to extract field name for checksum calculation"
            );
        }
    }
    errors.into_result()?;

    let Some(trigger_entity) = struct_options.trigger_entity else {
        bail!(input, "MV must have a trigger_entity attribute");
    };
    let Some(self_reference_kind) = struct_options.reference_kind else {
        bail!(input, "MV must have a reference_kind attribute");
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

    let mut sorted_reference_kinds: Vec<Path> = reference_kinds.into_iter().collect();
    sorted_reference_kinds.sort_by_cached_key(path_to_string);

    let checksum_static_ident = format_ident!(
        "{}_MATERIALIZED_VIEW_DEFINITION_CHECKSUM",
        ident.to_string().to_uppercase()
    );

    let output = quote! {
        impl crate::MaterializedView for #ident {
            fn kind() -> crate::reference::ReferenceKind {
                #self_reference_kind
            }

            fn reference_dependencies() -> &'static [crate::reference::ReferenceKind] {
                &[#(#sorted_reference_kinds),*]
            }

            fn trigger_entity() -> ::si_events::workspace_snapshot::EntityKind {
                #trigger_entity
            }

            fn definition_checksum() -> ::si_events::workspace_snapshot::Checksum {
                *#checksum_static_ident
            }
        }

        static #checksum_static_ident: ::std::sync::LazyLock<::si_events::workspace_snapshot::Checksum> =
            ::std::sync::LazyLock::new(|| {
                #definition_checksum
            });

        ::inventory::submit! {
            crate::materialized_view::MaterializedViewInventoryItem::new(
                #self_reference_kind,
                &[#(#sorted_reference_kinds),*],
                #trigger_entity,
                &#checksum_static_ident,
            )
        };
    };

    Ok(output.into())
}
