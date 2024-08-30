use traits::AnyNodeWeight;
use crate::{workspace_snapshot::node_weight::*, NodeWeightDiscriminants};

// Only for specific subtypes of NodeWeight
pub trait DiscriminatedNodeWeight: AnyNodeWeight
    where for<'de> Self: serde::Deserialize<'de>,
{
    const DISCRIMINANT: NodeWeightDiscriminants;
}

macro_rules! impl_discriminated_node_weight {
    ( $( $discriminant:ident($type:ty)),* ) => {$(
        impl $crate::workspace_snapshot::node_weight::AnyNodeWeight for $type {
            fn id(&self) -> si_events::ulid::Ulid { self.id }
            fn lineage_id(&self) -> $crate::workspace_snapshot::graph::LineageId { self.lineage_id }
            fn set_id_and_lineage(&mut self, id: impl Into<si_events::ulid::Ulid>, lineage_id: $crate::workspace_snapshot::graph::LineageId) {
                self.id = id.into();
                self.lineage_id = lineage_id;
            }
        
            fn merkle_tree_hash(&self) -> si_events::merkle_tree_hash::MerkleTreeHash { self.merkle_tree_hash }
            fn set_merkle_tree_hash(&mut self, new_hash: si_events::merkle_tree_hash::MerkleTreeHash) {
                self.merkle_tree_hash = new_hash;
            }
        }
        impl $crate::workspace_snapshot::node_weight::DiscriminatedNodeWeight for $type {
            const DISCRIMINANT: $crate::NodeWeightDiscriminants = $crate::NodeWeightDiscriminants::$discriminant;
        }
    )*};
}

pub(crate) use impl_discriminated_node_weight;