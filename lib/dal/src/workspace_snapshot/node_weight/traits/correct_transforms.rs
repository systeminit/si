use crate::{
    workspace_snapshot::{graph::{detect_updates::Update, WorkspaceSnapshotGraphError}, node_weight::NodeWeightDiscriminants},
    WorkspaceSnapshotGraphV2,
};
use thiserror::Error;


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
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        Ok(updates)
    }
}
