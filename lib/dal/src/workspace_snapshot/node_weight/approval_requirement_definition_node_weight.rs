use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    ulid::Ulid,
};

use super::traits::SiVersionedNodeWeight;

pub mod v1;

pub use v1::ApprovalRequirementDefinitionNodeWeightV1;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight, Hash,
)]
pub enum ApprovalRequirementDefinitionNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(ApprovalRequirementDefinitionNodeWeightV1),
}

impl ApprovalRequirementDefinitionNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self::V1(ApprovalRequirementDefinitionNodeWeightV1::new(
            id,
            lineage_id,
            content_hash,
        ))
    }
}
