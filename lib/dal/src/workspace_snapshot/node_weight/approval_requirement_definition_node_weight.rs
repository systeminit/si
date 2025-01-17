use serde::{Deserialize, Serialize};
use si_events::{ulid::Ulid, ContentHash};
use thiserror::Error;

use super::traits::SiVersionedNodeWeight;

pub mod v1;

pub use v1::ApprovalRequirementDefinitionNodeWeightV1;

// TODO(nick): use this or delete it.
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ApprovalRequirementNodeWeightError {
    // #[error("Invalid content for node weight: {0}")]
    // InvalidContentForNodeWeight(Ulid),
    // #[error("LayerDb error: {0}")]
    // LayerDb(#[from] LayerDbError),
    // #[error("NodeWeight error: {0}")]
    // NodeWeight(#[from] Box<NodeWeightError>),
    // #[error("WorkspaceSnapshot error: {0}")]
    // WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    // #[error("WorkspaceSnapshotGraph error: {0}")]
    // WorkspaceSnapshotGraph(#[from] Box<WorkspaceSnapshotGraphError>),
}

// TODO(nick): use this or delete it.
#[allow(dead_code)]
type Result<T> = std::result::Result<T, ApprovalRequirementNodeWeightError>;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight,
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
