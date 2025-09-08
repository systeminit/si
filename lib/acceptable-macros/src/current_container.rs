use std::str::FromStr;

use darling::FromAttributes;
use manyhow::{
    Result,
    bail,
    emit,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
};
use syn::{
    Data,
    DataEnum,
    DeriveInput,
    Error,
    Fields,
    Ident,
    Type,
    Visibility,
};

#[derive(Debug, FromAttributes)]
#[darling(attributes(acceptable))]
struct Config {}

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(acceptable))]
struct VariantConfig {
    current: Option<()>,
}

pub(crate) fn expand(input: TokenStream, errors: &mut manyhow::Emitter) -> Result<TokenStream2> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data,
        attrs,
        vis,
        ..
    } = input;
    let config = Config::from_attributes(&attrs)?;

    match data {
        Data::Enum(data_enum) => derive_from_enum(ident, data_enum, vis, config, errors),
        Data::Struct(_) | Data::Union(_) => {
            bail!("current only enum types are supported for `#[derive(AllVersions)]`")
        }
    }
}

fn derive_from_enum(
    ident: Ident,
    data: DataEnum,
    vis: Visibility,
    _config: Config,
    errors: &mut manyhow::Emitter,
) -> Result<TokenStream2> {
    let mut versions = Vec::with_capacity(data.variants.len());

    let mut current: Option<(Ident, Type, u64)> = None;

    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        if !variant_name.starts_with('V') {
            emit!(
                errors,
                Error::new_spanned(variant, "variant name must begin with 'V'")
            );
            continue;
        }

        let Ok(variant_version) = u64::from_str(&variant_name[1..]) else {
            emit!(
                errors,
                Error::new_spanned(variant, "variant name must end with a u64 version number")
            );
            continue;
        };

        let variant_ty = match &variant.fields {
            Fields::Unnamed(fields_unnamed) => {
                let mut iter = fields_unnamed.unnamed.iter();
                match (iter.next(), iter.next()) {
                    (Some(field), None) => &field.ty,

                    (None, None) | (None, Some(_)) | (Some(_), Some(_)) => {
                        emit!(
                            errors,
                            Error::new_spanned(
                                variant,
                                "only 1-tuple style variants are supported"
                            )
                        );
                        continue;
                    }
                }
            }
            Fields::Named(_) | Fields::Unit => {
                emit!(
                    errors,
                    Error::new_spanned(variant, "only tuple style variants are supported")
                );
                continue;
            }
        };

        let variant_config = VariantConfig::from_attributes(&variant.attrs)?;

        if variant_config.current.is_some() {
            if current.is_some() {
                emit!(
                    errors,
                    Error::new_spanned(
                        &ident,
                        "only one variant can have `#[acceptable(current)]` set"
                    )
                );
                continue;
            }

            current = Some((variant_ident.clone(), variant_ty.clone(), variant_version))
        }

        versions.push(variant_version);
    }

    errors.into_result()?;

    let Some((current_variant_ident, current_variant_ty, _current_version)) = current else {
        bail!(
            ident,
            "one variant must be marked with `#[acceptable(current)]`",
        );
    };

    let ident_name = ident.to_string();
    let container_ty_name = ident_name
        .strip_suffix("AllVersions")
        .unwrap_or(&ident_name);
    let container_ty_ident = format_ident!("{container_ty_name}");

    let current_alias_ty = format_ident!("{container_ty_name}VCurrent");

    Ok(quote! {
        #vis type #current_alias_ty = #current_variant_ty;

        #[derive(serde::Serialize)]
        #[derive(acceptable::Container)]
        #[derive(Clone, Eq, PartialEq)]
        #[acceptable(all_versions = #ident)]
        #[serde(rename_all = "camelCase")]
        #vis enum #container_ty_ident {
            #current_variant_ident(#current_variant_ty)
        }

        impl std::fmt::Debug for #container_ty_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::#current_variant_ident(inner) => {
                        f.debug_tuple(#container_ty_name).field(inner).finish()
                    }
                }
            }
        }

        impl std::ops::Deref for #container_ty_ident {
            type Target = #current_alias_ty;

            fn deref(&self) -> &Self::Target {
                match self {
                    Self::#current_variant_ident(inner) => inner,
                }
            }
        }

        impl std::ops::DerefMut for #container_ty_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match self {
                    Self::#current_variant_ident(inner) => inner,
                }
            }
        }
    })
}
