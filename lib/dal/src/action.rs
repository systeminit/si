use serde::{Deserialize, Serialize};
use si_events::ulid::Ulid;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    id, implement_add_edge_to,
    workspace_snapshot::node_weight::{
        category_node_weight::CategoryNodeKind, ActionNodeWeight, NodeWeight, NodeWeightError,
    },
    ChangeSetError, ChangeSetId, ComponentId, DalContext, EdgeWeightError, EdgeWeightKind,
    EdgeWeightKindDiscriminants, HelperError, TransactionsError, WorkspaceSnapshotError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionError {
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

pub type ActionResult<T> = Result<T, ActionError>;

id!(ActionId);
id!(ActionPrototypeId);

#[derive(Debug, Copy, Clone, Deserialize, Serialize, EnumDiscriminants, PartialEq, Eq)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize))]
pub enum ActionState {
    /// Action has been determined to be eligible to run, and has had its job sent to the job
    /// queue.
    Dispatched,
    /// Action failed during execution. See the job history for details.
    Failed,
    /// Action is "queued", but should not be considered as eligible to run, until moved to the
    /// `Queued` state.
    OnHold,
    /// Action is available to be dispatched once all of its prerequisites have succeeded, and been
    /// removed from the graph.
    Queued,
    /// Action has been dispatched, and started execution in the job system. See the job history
    /// for details.
    Running,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Action {
    id: ActionId,
    state: ActionState,
    originating_changeset_id: ChangeSetId,
}

impl From<ActionNodeWeight> for Action {
    fn from(value: ActionNodeWeight) -> Self {
        Self {
            id: value.id().into(),
            state: value.state(),
            originating_changeset_id: value.originating_changeset_id(),
        }
    }
}

impl Action {
    pub fn id(&self) -> ActionId {
        self.id
    }

    implement_add_edge_to!(
        source_id: ActionId,
        destination_id: ComponentId,
        add_fn: add_edge_to_component,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ActionResult,
    );
    // Even though we're using `implement_add_edge_to`, we're not creating an edge from Self *TO*
    // the Category node. We're adding an edge *FROM* the Category node to Self.
    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: ActionId,
        add_fn: add_incoming_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ActionResult,
    );

    pub async fn new(
        ctx: &DalContext,
        _action_prototype_id: &ActionPrototypeId,
        maybe_component_id: Option<ComponentId>,
    ) -> ActionResult<Self> {
        let change_set = ctx.change_set()?;
        let new_id: ActionId = change_set.generate_ulid()?.into();
        let node_weight = NodeWeight::new_action(change_set, new_id.into())?;
        ctx.workspace_snapshot()?.add_node(node_weight).await?;

        let action_category_id = ctx
            .workspace_snapshot()?
            .get_category_node(None, CategoryNodeKind::Action)
            .await?;
        Self::add_incoming_category_edge(
            ctx,
            action_category_id,
            new_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        // TODO: Add edge to the ActionPrototype

        if let Some(component_id) = maybe_component_id {
            Self::add_edge_to_component(ctx, new_id, component_id, EdgeWeightKind::new_use())
                .await?;
        }

        let new_action: Self = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(new_id)
            .await?
            .get_action_node_weight()?
            .into();

        Ok(new_action)
    }
}
