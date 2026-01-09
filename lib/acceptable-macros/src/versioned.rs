use darling::FromAttributes;
use manyhow::{
    Result,
    bail,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data,
    DataEnum,
    DataStruct,
    DeriveInput,
    Fields,
    Ident,
};

#[derive(Debug, FromAttributes)]
#[darling(attributes(acceptable))]
struct Config {
    version: u64,
}

pub(crate) fn expand(input: TokenStream, errors: &mut manyhow::Emitter) -> Result<TokenStream2> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident, data, attrs, ..
    } = input;
    let config = Config::from_attributes(&attrs)?;

    match data {
        Data::Struct(data_struct) => derive_from_struct(ident, data_struct, config, errors),
        Data::Enum(data_enum) => derive_from_enum(ident, data_enum, config, errors),
        Data::Union(_) => {
            bail!("union types are not supported for `#[derive(Versioned)]`")
        }
    }
}

fn derive_from_struct(
    ident: Ident,
    data: DataStruct,
    config: Config,
    _errors: &mut manyhow::Emitter,
) -> Result<TokenStream2> {
    let version = config.version;
    let Some(id_field) = data
        .fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .find(|ident| *ident == "id")
    else {
        bail!(ident, "unable to find 'id' field in struct");
    };

    Ok(quote! {
        impl acceptable::Versioned for #ident {
            #[inline]
            fn id(&self) -> acceptable::RequestId {
                self.#id_field.into()
            }

            #[inline]
            fn message_version() -> u64 {
                #version
            }
        }
    })
}

fn derive_from_enum(
    ident: Ident,
    data: DataEnum,
    config: Config,
    _errors: &mut manyhow::Emitter,
) -> Result<TokenStream2> {
    let version = config.version;

    // Verify that every variant has an `id` field
    let mut match_arms = Vec::new();
    for variant in &data.variants {
        let variant_ident = &variant.ident;

        let has_id_field = match &variant.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|field| field.ident.as_ref())
                .any(|ident| ident == "id"),
            Fields::Unnamed(_) => false,
            Fields::Unit => false,
        };

        if !has_id_field {
            bail!(
                variant_ident,
                "enum variant `{}` must have an `id` field for versioning",
                variant_ident
            );
        }

        match_arms.push(quote! {
            Self::#variant_ident { id, .. } => (*id).into()
        });
    }

    Ok(quote! {
        impl acceptable::Versioned for #ident {
            #[inline]
            fn id(&self) -> acceptable::RequestId {
                match self {
                    #(#match_arms),*
                }
            }

            #[inline]
            fn message_version() -> u64 {
                #version
            }
        }
    })
}
