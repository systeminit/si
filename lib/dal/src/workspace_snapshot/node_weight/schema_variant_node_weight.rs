use anyhow::Result;
use serde::{Deserialize, Serialize};
use si_events::{ulid::Ulid, ContentHash};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use super::{traits::SiVersionedNodeWeight, ContentNodeWeight, NodeWeightError};
use crate::{
    workspace_snapshot::graph::WorkspaceSnapshotGraphError, DalContext, WorkspaceSnapshotError,
};

pub mod v1;

use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphV3;
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

pub type SchemaVariantNodeWeightResult<T> = Result<T>;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiVersionedNodeWeight,
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

    pub async fn try_upgrade_from_content_node_weight(
        ctx: &DalContext,
        v3_graph: &mut WorkspaceSnapshotGraphV3,
        content_node_weight: &ContentNodeWeight,
    ) -> SchemaVariantNodeWeightResult<()> {
        SchemaVariantNodeWeightV1::try_upgrade_from_content_node_weight(
            ctx,
            v3_graph,
            content_node_weight,
        )
        .await
    }
}
