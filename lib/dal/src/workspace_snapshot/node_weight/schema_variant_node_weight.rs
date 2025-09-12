use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    ulid::Ulid,
};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use super::{
    NodeWeightError,
    traits::SiVersionedNodeWeight,
};
use crate::{
    WorkspaceSnapshotError,
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
};

pub mod v1;

pub use v1::SchemaVariantNodeWeightV1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantNodeWeightError {
    #[error("Invalid content for node weight: {0}")]
    InvalidContentForNodeWeight(Ulid),
    #[error("LayerDb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] Box<NodeWeightError>),
    #[error("WorkspaceSnapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("WorkspaceSnapshotGraph error: {0}")]
    WorkspaceSnapshotGraph(#[from] Box<WorkspaceSnapshotGraphError>),
}

pub type SchemaVariantNodeWeightResult<T> = Result<T, SchemaVariantNodeWeightError>;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight, Hash,
)]
pub enum SchemaVariantNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(SchemaVariantNodeWeightV1),
}

impl SchemaVariantNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, is_locked: bool, content_hash: ContentHash) -> Self {
        Self::V1(SchemaVariantNodeWeightV1::new(
            id,
            lineage_id,
            is_locked,
            content_hash,
        ))
    }
}
