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

pub use v1::ManagementPrototypeNodeWeightV1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ManagementPrototypeNodeWeightError {
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

pub type ManagementPrototypeNodeWeightResult<T> = Result<T, ManagementPrototypeNodeWeightError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ManagementPrototypeNodeWeight {
    V1(ManagementPrototypeNodeWeightV1),
}

impl ManagementPrototypeNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self::V1(ManagementPrototypeNodeWeightV1::new(
            id,
            lineage_id,
            content_hash,
        ))
    }
}

impl SiVersionedNodeWeight for ManagementPrototypeNodeWeight {
    type Inner = ManagementPrototypeNodeWeightV1;

    /// Return a reference to the most up to date enum variant
    fn inner(&self) -> &ManagementPrototypeNodeWeightV1 {
        match self {
            ManagementPrototypeNodeWeight::V1(inner) => inner,
        }
    }

    /// Return a mutable reference to the most up to date enum variant
    fn inner_mut(&mut self) -> &mut ManagementPrototypeNodeWeightV1 {
        match self {
            ManagementPrototypeNodeWeight::V1(inner) => inner,
        }
    }
}
