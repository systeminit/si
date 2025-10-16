use std::convert::AsRef;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    NodeWeightDiscriminants,
    func::leaf::LeafKind,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::traits::{
            CorrectExclusiveOutgoingEdge,
            CorrectTransforms,
            SiNodeWeight,
        },
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiNodeWeight, Hash)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::ApprovalRequirementDefinition)]
pub struct LeafPrototypeNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    // This points to a vec of AttributePath objects
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    pub(crate) content_address: ContentAddress,
    #[si_node_weight(node_hash = "self.kind.as_ref().as_bytes()")]
    pub(super) kind: LeafKind,
    timestamp: Timestamp,
}

impl LeafPrototypeNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, kind: LeafKind, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            kind,
            content_address: ContentAddress::AttributePaths(content_hash),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::AttributePaths(new_content_hash);
    }
}

impl CorrectTransforms for LeafPrototypeNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for LeafPrototypeNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[crate::EdgeWeightKindDiscriminants] {
        &[]
    }
}
