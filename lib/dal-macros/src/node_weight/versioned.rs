use darling::FromAttributes;
use manyhow::{bail, emit};
use quote::quote;
use syn::{Data, DeriveInput, Fields};

#[derive(FromAttributes, Default)]
#[darling(default, attributes(si_versioned_node_weight))]
struct SiVersionedNodeWeightVariantOptions {
    current: bool,
}

pub fn derive_si_versioned_node_weight(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        ..
    } = input.clone();

    let enum_data = match &type_data {
        Data::Enum(enum_data) => enum_data,
        _ => {
            bail!(input, "SiVersionedNodeWeight must be derived on an enum.");
        }
    };

    let mut maybe_current_variant = None;
    let mut maybe_inner_type = None;
    for variant in &enum_data.variants {
        let variant_attrs = SiVersionedNodeWeightVariantOptions::from_attributes(&variant.attrs)?;
        if variant_attrs.current {
            if maybe_current_variant.is_none() {
                maybe_current_variant = Some(variant.ident.clone());
            } else {
                emit!(
                    errors,
                    &variant,
                    "Enum cannot have multiple current variants."
                );
            }
        }
        if variant_attrs.current {
            match &variant.fields {
                Fields::Named(_) => {
                    emit!(
                        errors,
                        variant.fields,
                        "Current variant must have a single unnamed field."
                    );
                    continue;
                }
                Fields::Unit => {
                    emit!(
                        errors,
                        variant,
                        "Current variant must have a single unnamed field."
                    );
                    continue;
                }
                Fields::Unnamed(_) => {}
            }
            for field in &variant.fields {
                if maybe_inner_type.is_none() {
                    maybe_inner_type = Some(field.ty.clone());
                }
            }
            if variant.fields.len() != 1 {
                emit!(
                    errors,
                    &variant.fields,
                    "Current variant must have a single unnamed field."
                );
            }
        }
    }
    errors.into_result()?;

    if maybe_current_variant.is_none() {
        emit!(errors, &input, "No current variant annotation");
    }
    if maybe_inner_type.is_none() && maybe_inner_type.is_some() {
        emit!(errors, &input, "No inner type found for current variant");
    }
    errors.into_result()?;

    let current_variant_ident = maybe_current_variant
        .expect("maybe_current_variant is None while Option::is_none() is false");
    let inner_type =
        maybe_inner_type.expect("maybe_inner_type is None while Option::is_none() is false");

    let output = quote! {
        impl SiVersionedNodeWeight for #ident {
            type Inner = #inner_type;

            /// Return a reference to the most up to date enum variant
            fn inner(&self) -> &Self::Inner {
                match self {
                    Self::#current_variant_ident(inner) => inner,
                    _ => {
                        panic!("Attempted to get reference to unsupported #ident variant");
                    }
                }
            }

            /// Return a mutable reference to the most up to date enum variant
            fn inner_mut(&mut self) -> &mut Self::Inner {
                match self {
                    Self::#current_variant_ident(inner) => inner,
                    _ => {
                        panic!("Attempted to get mutable reference to unsupported #ident variant");
                    }
                }
            }
        }
    };

    Ok(output.into())
}
