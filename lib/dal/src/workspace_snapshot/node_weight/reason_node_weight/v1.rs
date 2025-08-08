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

use super::Reason;
use crate::{
    NodeWeightDiscriminants,
    workspace_snapshot::{
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
pub struct ReasonNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub reason: Reason,
    merkle_tree_hash: MerkleTreeHash,
    timestamp: Timestamp,
}

impl ReasonNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, reason: Reason) -> Self {
        Self {
            id,
            lineage_id,
            reason,
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }
}

impl CorrectTransforms for ReasonNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for ReasonNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[crate::EdgeWeightKindDiscriminants] {
        &[]
    }
}
