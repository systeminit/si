use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::ulid::Ulid;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    action::dependency_graph::ActionDependencyGraph,
    id, implement_add_edge_to,
    workspace_snapshot::node_weight::{
        category_node_weight::CategoryNodeKind, ActionNodeWeight, NodeWeight, NodeWeightError,
    },
    ChangeSetError, ChangeSetId, ComponentError, ComponentId, DalContext, EdgeWeightError,
    EdgeWeightKind, EdgeWeightKindDiscriminants, HelperError, TransactionsError,
    WorkspaceSnapshotError,
};

pub mod dependency_graph;
pub mod prototype;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionError {
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Component error: {0}")]
    Component(#[from] ComponentError),
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
    implement_add_edge_to!(
        source_id: ActionId,
        destination_id: ActionPrototypeId,
        add_fn: add_edge_to_action_prototype,
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
        action_prototype_id: ActionPrototypeId,
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

        Self::add_edge_to_action_prototype(
            ctx,
            new_id,
            action_prototype_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

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

    /// Sort the dependency graph of [`Actions`][Action] topologically, breaking ties by listing
    /// [`Actions`][Action] sorted by their ID (oldest first thanks to ULID sorting).
    pub async fn list_topologically(ctx: &DalContext) -> ActionResult<Vec<ActionId>> {
        // TODO: Grab all "running" & "failed" Actions to list first?
        let mut result = Vec::new();

        let mut action_dependency_graph = ActionDependencyGraph::for_workspace(ctx).await?;

        loop {
            let mut independent_actions = action_dependency_graph.independent_actions();
            if independent_actions.is_empty() {
                break;
            }

            independent_actions.sort();
            for action_id in independent_actions {
                action_dependency_graph.remove_action(action_id);
                result.push(action_id);
            }
        }

        // If there is a cycle in the dependencies for some reason, we still want to know that the
        // actions exist, even though they'll never start executing.
        let mut actions_in_cycle = action_dependency_graph.remaining_actions();
        actions_in_cycle.sort();
        result.extend(&actions_in_cycle);

        Ok(result)
    }

    pub async fn component_id(
        ctx: &DalContext,
        action_id: ActionId,
    ) -> ActionResult<Option<ComponentId>> {
        for (_, _tail_node_idx, head_node_idx) in ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                action_id,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            if let NodeWeight::Component(component_node_weight) = ctx
                .workspace_snapshot()?
                .get_node_weight(head_node_idx)
                .await?
            {
                return Ok(Some(component_node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub async fn all_ids(ctx: &DalContext) -> ActionResult<Vec<ActionId>> {
        let mut result = Vec::new();

        let action_category_node_index = ctx
            .workspace_snapshot()?
            .get_category_node(None, CategoryNodeKind::Action)
            .await?;
        for (_, _, action_node_index) in ctx
            .workspace_snapshot()?
            .edges_directed(action_category_node_index, Outgoing)
            .await?
        {
            let action_id = ctx
                .workspace_snapshot()?
                .get_node_weight(action_node_index)
                .await?
                .id();
            result.push(action_id.into());
        }

        Ok(result)
    }
}
