use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::ulid::Ulid;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::ResourceStatus;

use crate::{
    action::dependency_graph::ActionDependencyGraph,
    action::prototype::{ActionKind, ActionPrototype, ActionPrototypeError},
    func::backend::js_action::DeprecatedActionRunResult,
    func::execution::{FuncExecution, FuncExecutionError, FuncExecutionPk},
    id, implement_add_edge_to,
    job::definition::ActionJob,
    workspace_snapshot::node_weight::{
        category_node_weight::CategoryNodeKind, ActionNodeWeight, NodeWeight, NodeWeightError,
    },
    ChangeSetError, ChangeSetId, ComponentError, ComponentId, DalContext, EdgeWeightError,
    EdgeWeightKind, EdgeWeightKindDiscriminants, HelperError, TransactionsError,
    WorkspaceSnapshotError, WsEvent, WsEventResult, WsPayload,
};

pub mod dependency_graph;
pub mod prototype;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found for action: {0}")]
    ComponentNotFoundForAction(ActionId),
    #[error("Edge Weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func execution error: {0}")]
    FuncExecution(#[from] FuncExecutionError),
    #[error("Helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("Node Weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("prototype not found for action: {0}")]
    PrototypeNotFoundForAction(ActionId),
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
    func_execution_pk: Option<FuncExecutionPk>,
}

impl From<ActionNodeWeight> for Action {
    fn from(value: ActionNodeWeight) -> Self {
        Self {
            id: value.id().into(),
            state: value.state(),
            originating_changeset_id: value.originating_changeset_id(),
            func_execution_pk: value.func_execution_pk(),
        }
    }
}

impl Action {
    pub fn id(&self) -> ActionId {
        self.id
    }

    pub fn state(&self) -> ActionState {
        self.state
    }

    pub fn originating_changeset_id(&self) -> ChangeSetId {
        self.originating_changeset_id
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

    pub async fn find_equivalent(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        maybe_component_id: Option<ComponentId>,
    ) -> ActionResult<Option<ActionId>> {
        let snap = ctx.workspace_snapshot()?;
        let action_category_id = snap
            .get_category_node(None, CategoryNodeKind::Action)
            .await?;

        for action_idx in snap
            .outgoing_targets_for_edge_weight_kind(
                action_category_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            let action_id: ActionId = snap
                .get_node_weight(action_idx)
                .await?
                .get_action_node_weight()?
                .id()
                .into();

            if Self::component_id(ctx, action_id).await? == maybe_component_id
                && Self::prototype_id(ctx, action_id).await? == action_prototype_id
            {
                // we found the equivalent!
                return Ok(Some(action_id));
            }
        }
        Ok(None)
    }

    pub async fn get_by_id(ctx: &DalContext, id: ActionId) -> ActionResult<Self> {
        let action: Self = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(id)
            .await?
            .get_action_node_weight()?
            .into();
        Ok(action)
    }

    pub async fn set_state(ctx: &DalContext, id: ActionId, state: ActionState) -> ActionResult<()> {
        let idx = ctx.workspace_snapshot()?.get_node_index_by_id(id).await?;
        let mut node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(id)
            .await?
            .get_action_node_weight()?;
        node_weight.set_state(state);
        ctx.workspace_snapshot()?
            .add_node(NodeWeight::Action(node_weight))
            .await?;
        ctx.workspace_snapshot()?.replace_references(idx).await?;
        Ok(())
    }

    pub async fn set_func_execution_pk(
        ctx: &DalContext,
        id: ActionId,
        pk: Option<FuncExecutionPk>,
    ) -> ActionResult<()> {
        let idx = ctx.workspace_snapshot()?.get_node_index_by_id(id).await?;
        let mut node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(id)
            .await?
            .get_action_node_weight()?;
        node_weight.set_func_execution_pk(pk);
        ctx.workspace_snapshot()?
            .add_node(NodeWeight::Action(node_weight))
            .await?;
        ctx.workspace_snapshot()?.replace_references(idx).await?;
        Ok(())
    }

    pub async fn new(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        maybe_component_id: Option<ComponentId>,
    ) -> ActionResult<Self> {
        let change_set = ctx.change_set()?;
        let new_id: ActionId = change_set.generate_ulid()?.into();
        let originating_change_set_id = ctx.change_set_id();
        let node_weight =
            NodeWeight::new_action(change_set, originating_change_set_id, new_id.into())?;
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

    pub async fn remove(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        maybe_component_id: Option<ComponentId>,
    ) -> ActionResult<()> {
        let change_set = ctx.change_set()?;
        let snap = ctx.workspace_snapshot()?;

        if let Some(action_id) =
            Self::find_equivalent(ctx, action_prototype_id, maybe_component_id).await?
        {
            let action_ulid: Ulid = action_id.into();
            snap.remove_node_by_id(change_set, action_ulid).await?;
        }
        Ok(())
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

    pub async fn prototype_id(
        ctx: &DalContext,
        action_id: ActionId,
    ) -> ActionResult<ActionPrototypeId> {
        for (_, _tail_node_idx, head_node_idx) in ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                action_id,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            if let NodeWeight::ActionPrototype(node_weight) = ctx
                .workspace_snapshot()?
                .get_node_weight(head_node_idx)
                .await?
            {
                return Ok(node_weight.id().into());
            }
        }

        Err(ActionError::PrototypeNotFoundForAction(action_id))
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

    pub async fn func_execution(&self, ctx: &DalContext) -> ActionResult<Option<FuncExecution>> {
        Ok(match self.func_execution_pk {
            Some(pk) => Some(FuncExecution::get_by_pk(ctx, &pk).await?),
            None => None,
        })
    }

    pub async fn run(
        ctx: &DalContext,
        id: ActionId,
    ) -> ActionResult<Option<DeprecatedActionRunResult>> {
        let component_id = Action::component_id(ctx, id)
            .await?
            .ok_or(ActionError::ComponentNotFoundForAction(id))?;

        let prototype_id = Action::prototype_id(ctx, id).await?;
        let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;

        let (func_execution_pk, resource) =
            ActionPrototype::run(ctx, prototype.id, component_id).await?;
        Action::set_func_execution_pk(ctx, id, Some(func_execution_pk)).await?;

        if matches!(
            resource.as_ref().and_then(|r| r.status),
            Some(ResourceStatus::Ok)
        ) {
            ctx.workspace_snapshot()?
                .remove_node_by_id(ctx.change_set()?, id)
                .await?;
        } else {
            Action::set_state(ctx, id, ActionState::Failed).await?;
        }

        Ok(resource)
    }

    pub async fn eligible_to_dispatch(ctx: &DalContext) -> ActionResult<Vec<ActionId>> {
        let action_dependency_graph = ActionDependencyGraph::for_workspace(ctx).await?;
        let mut result = Vec::new();

        for possible_action_id in action_dependency_graph.independent_actions() {
            let action = Action::get_by_id(ctx, possible_action_id).await?;

            // Only Actions in the ActionState::Queued state are dispatchable.
            if action.state() == ActionState::Queued {
                result.push(possible_action_id);
            }
        }

        Ok(result)
    }

    pub async fn dispatch_action(ctx: &DalContext, action_id: ActionId) -> ActionResult<()> {
        Action::set_state(ctx, action_id, ActionState::Dispatched).await?;

        ctx.enqueue_action(ActionJob::new(ctx, action_id)).await?;

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionReturn {
    id: ActionId,
    component_id: ComponentId,
    kind: ActionKind,
    resource: Option<DeprecatedActionRunResult>,
}

impl WsEvent {
    pub async fn action_return(
        ctx: &DalContext,
        id: ActionId,
        kind: ActionKind,
        component_id: ComponentId,
        resource: Option<DeprecatedActionRunResult>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ActionReturn(ActionReturn {
                id,
                kind,
                component_id,
                resource,
            }),
        )
        .await
    }
}
