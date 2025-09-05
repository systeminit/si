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
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::traits::{
            CorrectExclusiveOutgoingEdge,
            CorrectTransforms,
            ExclusiveOutgoingEdges,
            SiNodeWeight,
        },
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiNodeWeight, Hash)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::ApprovalRequirementDefinition)]
pub struct ApprovalRequirementDefinitionNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl ApprovalRequirementDefinitionNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            content_address: ContentAddress::ApprovalRequirementDefinition(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::ApprovalRequirementDefinition(new_content_hash);
    }
}

impl CorrectTransforms for ApprovalRequirementDefinitionNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for ApprovalRequirementDefinitionNodeWeightV1 {}

impl ExclusiveOutgoingEdges for ApprovalRequirementDefinitionNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[crate::EdgeWeightKindDiscriminants] {
        &[]
    }
}
