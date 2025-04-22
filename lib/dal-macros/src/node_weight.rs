use std::collections::HashSet;

use darling::{
    FromAttributes,
    FromMeta,
    util::IdentString,
};
use manyhow::{
    bail,
    emit,
};
use proc_macro2::TokenStream;
use quote::{
    ToTokens,
    quote,
};
use syn::{
    Data,
    DeriveInput,
    Expr,
    Path,
};

pub(crate) mod versioned;

#[derive(Debug, PartialEq, Eq, Hash)]
enum SkipOption {
    ContentHash,
    Id,
    LineageId,
    MerkleTreeHash,
    NodeHash,
    NodeWeightDiscriminants,
    SetId,
    SetLineageId,
    SetMerkleTreeHash,
}

impl FromMeta for SkipOption {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "content_hash" => Ok(Self::ContentHash),
            "id" => Ok(Self::Id),
            "lineage_id" => Ok(Self::LineageId),
            "merkle_tree_hash" => Ok(Self::MerkleTreeHash),
            "node_hash" => Ok(Self::NodeHash),
            "node_weight_discriminants" => Ok(Self::NodeWeightDiscriminants),
            "set_id" => Ok(Self::SetId),
            "set_lineage_id" => Ok(Self::SetLineageId),
            "set_merkle_tree_hash" => Ok(Self::SetMerkleTreeHash),
            v => Err(darling::Error::unknown_value(v)),
        }
    }
}

#[derive(Debug, Default)]
struct SkipOptionSet {
    options: HashSet<SkipOption>,
}

impl SkipOptionSet {
    fn should_skip(&self, option: SkipOption) -> bool {
        self.options.contains(&option)
    }

    fn add_skip_option(&mut self, option: SkipOption) {
        self.options.insert(option);
    }
}

impl FromMeta for SkipOptionSet {
    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let mut new_option_set = SkipOptionSet::default();
        for item in items {
            new_option_set.add_skip_option(SkipOption::from_nested_meta(item)?);
        }

        Ok(new_option_set)
    }
}

#[derive(FromAttributes)]
#[darling(attributes(si_node_weight))]
struct SiNodeWeightOptions {
    discriminant: Option<Path>,
    #[darling(default)]
    skip: SkipOptionSet,
}

#[derive(Debug, Default)]
struct NodeHashParticipation {
    participant: bool,
    custom_code: Option<String>,
}

impl FromMeta for NodeHashParticipation {
    fn from_word() -> darling::Result<Self> {
        Ok(Self {
            participant: true,
            custom_code: None,
        })
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        match value {
            syn::Lit::Str(custom_str) => Ok(Self {
                participant: true,
                custom_code: Some(custom_str.value()),
            }),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
    }
}

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(si_node_weight))]
struct NodeWeightFieldAttrs {
    #[darling(default, rename = "node_hash")]
    node_hash_participation: NodeHashParticipation,
}

pub fn derive_si_node_weight(
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
    let struct_options = SiNodeWeightOptions::from_attributes(&attrs)?;

    let node_weight_discriminant = if let Some(discriminant) = struct_options.discriminant {
        discriminant
    } else {
        emit!(errors, input, "No NodeWeightDiscriminants was specified.");
        syn::parse_str::<Path>("")?
    };

    let struct_data = match &type_data {
        Data::Struct(data) => data,
        _ => {
            bail!(input, "SiNodeWeight must be derived on a struct.");
        }
    };
    let mut struct_has_content_address = false;
    let mut node_hash_parts = Vec::new();
    for field in &struct_data.fields {
        if let Some(field_ident) = &field.ident {
            let ident_string: IdentString = field_ident.clone().into();
            if ident_string == "content_address" {
                struct_has_content_address = true;
            }
        } else {
            emit!(errors, field, "No identifier found for field.");
            continue;
        }
        let field_attrs = NodeWeightFieldAttrs::from_attributes(&field.attrs)?;
        if field_attrs.node_hash_participation.participant {
            let hasher_update =
                if let Some(custom_code) = field_attrs.node_hash_participation.custom_code {
                    match syn::parse_str::<Expr>(&custom_code) {
                        Ok(expr) => expr.to_token_stream(),
                        Err(e) => {
                            emit!(
                                errors,
                                syn::Error::new_spanned(
                                    field,
                                    format!("Invalid custom code for node_hash calculation: {e}")
                                )
                            );
                            continue;
                        }
                    }
                } else {
                    let field_ident = field.ident.clone();
                    quote! {
                        self.#field_ident.as_bytes()
                    }
                };
            node_hash_parts.push(hasher_update);
        }
    }
    errors.into_result()?;

    let id_fn = if struct_options.skip.should_skip(SkipOption::Id) {
        quote! {}
    } else {
        quote! {
            fn id(&self) -> Ulid {
                self.id
            }
        }
    };

    let lineage_id_fn = if struct_options.skip.should_skip(SkipOption::LineageId) {
        quote! {}
    } else {
        quote! {
            fn lineage_id(&self) -> Ulid {
                self.lineage_id
            }
        }
    };

    let content_hash_fn = if struct_options.skip.should_skip(SkipOption::ContentHash) {
        quote! {}
    } else {
        let content_hash_location = if struct_has_content_address {
            quote! { self.content_address.content_hash() }
        } else {
            quote! { self.node_hash() }
        };

        quote! {
            fn content_hash(&self) -> ContentHash {
                #content_hash_location
            }
        }
    };

    let merkle_tree_hash_fn = if struct_options.skip.should_skip(SkipOption::MerkleTreeHash) {
        quote! {}
    } else {
        quote! {
            fn merkle_tree_hash(&self) -> MerkleTreeHash {
                self.merkle_tree_hash
            }
        }
    };

    let node_hash_fn = if struct_options.skip.should_skip(SkipOption::NodeHash) {
        quote! {}
    } else {
        let mut hash_updates = TokenStream::new();
        for update in node_hash_parts {
            let full_update = quote! { content_hasher.update(#update); };
            hash_updates.extend(full_update);
        }

        quote! {
            fn node_hash(&self) -> ContentHash {
                let mut content_hasher = ContentHash::hasher();
                #hash_updates

                content_hasher.finalize()
            }
        }
    };

    let node_weight_discriminant_fn = if struct_options
        .skip
        .should_skip(SkipOption::NodeWeightDiscriminants)
    {
        quote! {}
    } else {
        quote! {
            fn node_weight_discriminant(&self) -> NodeWeightDiscriminants {
                #node_weight_discriminant
            }
        }
    };

    let set_id_fn = if struct_options.skip.should_skip(SkipOption::SetId) {
        quote! {}
    } else {
        quote! {
            fn set_id(&mut self, new_id: Ulid) {
                self.id = new_id;
            }
        }
    };

    let set_lineage_id_fn = if struct_options.skip.should_skip(SkipOption::SetLineageId) {
        quote! {}
    } else {
        quote! {
            fn set_lineage_id(&mut self, new_lineage_id: Ulid) {
                self.lineage_id = new_lineage_id;
            }
        }
    };

    let set_merkle_tree_hash_fn = if struct_options
        .skip
        .should_skip(SkipOption::SetMerkleTreeHash)
    {
        quote! {}
    } else {
        quote! {
            fn set_merkle_tree_hash(&mut self, new_merkle_tree_hash: MerkleTreeHash) {
                self.merkle_tree_hash = new_merkle_tree_hash;
            }
        }
    };

    let output = quote! {
        impl SiNodeWeight for #ident {
            #id_fn
            #lineage_id_fn
            #content_hash_fn
            #merkle_tree_hash_fn
            #node_hash_fn
            #node_weight_discriminant_fn
            #set_id_fn
            #set_lineage_id_fn
            #set_merkle_tree_hash_fn
        }
    };

    Ok(output.into())
}
