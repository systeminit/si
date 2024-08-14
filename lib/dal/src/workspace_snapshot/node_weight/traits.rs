use crate::{
    workspace_snapshot::graph::{detect_updates::Update, WorkspaceSnapshotGraphError},
    WorkspaceSnapshotGraphV2,
};
use thiserror::Error;

use super::NodeWeightDiscriminants;

pub mod correct_exclusive_outgoing_edge;

pub use correct_exclusive_outgoing_edge::CorrectExclusiveOutgoingEdge;

#[derive(Debug, Error)]
pub enum CorrectTransformsError {
    #[error("expected a node weight of kind {0} but got another, or none")]
    UnexpectedNodeWeight(NodeWeightDiscriminants),
    #[error("workspace snapshot graph: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
}

pub type CorrectTransformsResult<T> = Result<T, CorrectTransformsError>;

pub trait CorrectTransforms {
    fn correct_transforms(
        &self,
        _workspace_snapshot_graph: &WorkspaceSnapshotGraphV2,
        updates: Vec<Update>,
    ) -> CorrectTransformsResult<Vec<Update>> {
        Ok(updates)
    }
}
