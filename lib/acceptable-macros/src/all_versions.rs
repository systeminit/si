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
};

#[derive(Debug, FromAttributes)]
#[darling(attributes(acceptable))]
struct Config {}

pub(crate) fn expand(input: TokenStream, errors: &mut manyhow::Emitter) -> Result<TokenStream2> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident, data, attrs, ..
    } = input;
    let config = Config::from_attributes(&attrs)?;

    match data {
        Data::Enum(data_enum) => derive_from_enum(ident, data_enum, config, errors),
        Data::Struct(_) | Data::Union(_) => {
            bail!("current only enum types are supported for `#[derive(AllVersions)]`")
        }
    }
}

fn derive_from_enum(
    ident: Ident,
    data: DataEnum,
    _config: Config,
    errors: &mut manyhow::Emitter,
) -> Result<TokenStream2> {
    let mut versions = Vec::with_capacity(data.variants.len());
    let mut variants = Vec::with_capacity(versions.len());

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

        let _variant_ty = match &variant.fields {
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

        versions.push(variant_version);
        variants.push(variant_ident);
    }

    errors.into_result()?;

    Ok(quote! {
        impl acceptable::AllVersions for #ident {
            #[inline]
            fn version(&self) -> u64 {
                match self {
                    #(Self::#variants(inner) => acceptable::Versioned::version(inner)),*
                }
            }

            #[inline]
            fn all_versions() -> &'static [u64] {
                &[#(#versions),*]
            }
        }
    })
}
