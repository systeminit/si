use serde::{Deserialize, Serialize};
use si_events::{ulid::Ulid, ContentHash};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use super::{traits::SiVersionedNodeWeight, ContentNodeWeight, NodeWeightError};
use crate::{
    workspace_snapshot::graph::WorkspaceSnapshotGraphError, DalContext, SocketArity,
    WorkspaceSnapshotError, WorkspaceSnapshotGraphV3,
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
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight,
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

    pub(crate) async fn try_upgrade_from_content_node_weight(
        ctx: &DalContext,
        v3_graph: &mut WorkspaceSnapshotGraphV3,
        content_node_weight: &ContentNodeWeight,
    ) -> InputSocketNodeWeightResult<()> {
        // InputSocketNodeWeightV1 is the first one not stored as a ContentNodeWeight, so is the
        // only one we can directly upgrade from a ContentNodeWeight.
        InputSocketNodeWeightV1::try_upgrade_from_content_node_weight(
            ctx,
            v3_graph,
            content_node_weight,
        )
        .await
    }
}
