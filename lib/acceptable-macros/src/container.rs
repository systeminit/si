use std::str::FromStr;

use darling::FromAttributes;
use manyhow::{
    Result,
    bail,
    emit,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data,
    DataEnum,
    DeriveInput,
    Error,
    Fields,
    Ident,
    Path,
    Type,
};

#[derive(Debug, FromAttributes)]
#[darling(attributes(acceptable))]
struct Config {
    all_versions: Path,
}

pub(crate) fn expand(input: TokenStream, errors: &mut manyhow::Emitter) -> Result<TokenStream2> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident, data, attrs, ..
    } = input;
    let config = Config::from_attributes(&attrs)?;

    match data {
        Data::Enum(data_enum) => derive_from_enum(ident, data_enum, config, errors),
        Data::Struct(_) | Data::Union(_) => {
            bail!("current only enum types are supported for `#[derive(Container)]`")
        }
    }
}

fn derive_from_enum(
    ident: Ident,
    data: DataEnum,
    config: Config,
    errors: &mut manyhow::Emitter,
) -> Result<TokenStream2> {
    if data.variants.len() != 1 {
        bail!(
            ident,
            "only one variant allowed in enums with `#[#[derive(Container)]]`",
        );
    }

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

        if current.is_some() {
            emit!(
                errors,
                Error::new_spanned(
                    &ident,
                    "only one variant allowed in enums with `#[#[derive(Container)]]`",
                )
            );
            continue;
        }

        current = Some((variant_ident.clone(), variant_ty.clone(), variant_version))
    }

    errors.into_result()?;

    let Some((current_variant_ident, current_variant_ty, _current_version)) = current else {
        bail!(
            ident,
            "one variant must be provided in enums with `#[#[derive(Container)]]`",
        );
    };

    let ident_name = ident.to_string();
    let all_versions_ty = config.all_versions;

    Ok(quote! {
        impl acceptable::Container for #ident {
            type Current = #current_variant_ty;

            type AllVersions = #all_versions_ty;

            const MESSAGE_TYPE: &'static str = #ident_name;

            #[inline]
            fn new(current: Self::Current) -> Self {
                Self::#current_variant_ident(current)
            }

            #[inline]
            fn id(&self) -> acceptable::RequestId {
                match self {
                    Self::#current_variant_ident(inner) => acceptable::Versioned::id(inner),
                }
            }

            #[inline]
            fn message_type() -> &'static str {
                Self::MESSAGE_TYPE
            }
        }
    })
}
