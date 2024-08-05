use crate::{workspace_snapshot::graph::detect_updates::Update, WorkspaceSnapshotGraphV2};
use thiserror::Error;

use super::NodeWeightDiscriminants;

#[derive(Debug, Error)]
pub enum CorrectTransformsError {
    #[error("expected a node weight of kind {0} but got another, or none")]
    UnexpectedNodeWeight(NodeWeightDiscriminants),
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
