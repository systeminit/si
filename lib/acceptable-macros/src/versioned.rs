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
    DataStruct,
    DeriveInput,
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
        Data::Enum(_) | Data::Union(_) => {
            bail!("current only struct types are supported for `#[derive(Versioned)]`")
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
