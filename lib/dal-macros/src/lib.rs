use manyhow::manyhow;

mod node_weight;

#[manyhow]
#[proc_macro_derive(SiVersionedNodeWeight, attributes(si_versioned_node_weight))]
pub fn si_versioned_node_weight(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    node_weight::versioned::derive_si_versioned_node_weight(input, errors)
}

#[manyhow]
#[proc_macro_derive(SiNodeWeight, attributes(si_node_weight))]
pub fn si_node_weight(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    node_weight::derive_si_node_weight(input, errors)
}
