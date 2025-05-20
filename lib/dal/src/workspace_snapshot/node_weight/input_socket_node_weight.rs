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
    SocketArity,
    WorkspaceSnapshotError,
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
};

pub mod v1;

pub use v1::InputSocketNodeWeightV1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InputSocketNodeWeightError {
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

pub type InputSocketNodeWeightResult<T> = Result<T, InputSocketNodeWeightError>;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight, Hash,
)]
pub enum InputSocketNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(InputSocketNodeWeightV1),
}

impl InputSocketNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, arity: SocketArity, content_hash: ContentHash) -> Self {
        Self::V1(InputSocketNodeWeightV1::new(
            id,
            lineage_id,
            arity,
            content_hash,
        ))
    }
}
