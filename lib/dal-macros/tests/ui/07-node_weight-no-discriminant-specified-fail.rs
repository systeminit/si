use dal::workspace_snapshot::{
    edge_weight::EdgeWeightKindDiscriminants,
    node_weight::{
        traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
        NodeWeightDiscriminants,
    },
};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

#[derive(dal_macros::SiNodeWeight)]
#[si_node_weight]
pub struct TestingNodeWeight {
    id: Ulid,
    lineage_id: Ulid,
    merkle_tree_hash: MerkleTreeHash,
}

impl CorrectTransforms for TestingNodeWeight {}
impl CorrectExclusiveOutgoingEdge for TestingNodeWeight {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

fn main() {}
