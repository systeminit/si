use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    implement_add_edge_to,
    workspace_snapshot::node_weight::{ActionPrototypeNodeWeight, NodeWeightError},
    ActionPrototypeId, ChangeSetError, DalContext, EdgeWeightError, EdgeWeightKindDiscriminants,
    FuncId, HelperError, TransactionsError, WorkspaceSnapshotError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionPrototypeError {
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Edge Weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("Helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("Node Weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("Workspace Snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ActionPrototypeResult<T> = Result<T, ActionPrototypeError>;

#[remain::sorted]
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum ActionKind {
    /// Create the "outside world" version of the modeled object.
    Create,
    /// Destroy the "outside world" version of the modeled object referenced in the resource.
    Destroy,
    /// This [`Action`][crate::Action] will only ever be manually queued.
    Manual,
    /// Refresh the resource to reflect the current state of the modeled object in the "outside
    /// world".
    Refresh,
    /// Update the version of the modeled object in the "outside world" to match the state of the
    /// model.
    Update,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionPrototype {
    id: ActionPrototypeId,
    kind: ActionKind,
    name: String,
    description: Option<String>,
}

impl From<ActionPrototypeNodeWeight> for ActionPrototype {
    fn from(value: ActionPrototypeNodeWeight) -> Self {
        Self {
            id: value.id().into(),
            kind: value.kind(),
            name: value.name().to_owned(),
            description: value.description().map(str::to_string),
        }
    }
}

impl ActionPrototype {
    pub fn id(&self) -> ActionPrototypeId {
        self.id
    }

    implement_add_edge_to!(
        source_id: ActionPrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ActionPrototypeResult,
    );
}
