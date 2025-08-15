use std::{
    collections::{
        HashSet,
        VecDeque,
    },
    time::Instant,
};

use itertools::Itertools;
use petgraph::prelude::*;
use postgres_types::{
    FromSql,
    ToSql,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    audit_log::AuditLogKind,
    ulid::Ulid,
};
use si_id::{
    FuncRunId,
    SchemaVariantId,
};
use si_layer_cache::LayerDbError;
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributeValue,
    ChangeSetError,
    ChangeSetId,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    Func,
    FuncError,
    HelperError,
    SchemaVariant,
    SchemaVariantError,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    WsEventResult,
    WsPayload,
    action::{
        dependency_graph::ActionDependencyGraph,
        prototype::{
            ActionKind,
            ActionPrototype,
            ActionPrototypeError,
        },
    },
    attribute::value::{
        AttributeValueError,
        DependentValueGraph,
    },
    component::inferred_connection_graph::InferredConnectionGraphError,
    func::FuncExecutionPk,
    implement_add_edge_to,
    workspace_snapshot::{
        DependentValueRoot,
        dependent_value_root::DependentValueRootError,
        node_weight::{
            ActionNodeWeight,
            NodeWeight,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
        },
    },
};

pub mod dependency_graph;
pub mod prototype;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionError {
    #[error("action already enqueued: {0}")]
    ActionAlreadyEnqueued(ActionPrototypeId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<crate::attribute::prototype::AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(
        #[from] Box<crate::attribute::prototype::argument::AttributePrototypeArgumentError>,
    ),
    #[error("AttributeValue error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("Component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("component not found for action: {0}")]
    ComponentNotFoundForAction(ActionId),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] Box<DependentValueRootError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("Helper error: {0}")]
    Helper(#[from] Box<HelperError>),
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] Box<InferredConnectionGraphError>),
    #[error("Layer DB error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("Node Weight error: {0}")]
    NodeWeight(#[from] Box<NodeWeightError>),
    #[error("prototype not found for action: {0}")]
    PrototypeNotFoundForAction(ActionId),
    #[error("Schema Variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("Transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("Unable to determine kind for action: {0}")]
    UnableToGetKind(ActionId),
    #[error("unexpected number of action kind {0} for variant {1}")]
    UnexpectedNumberOfActionKinds(ActionKind, SchemaVariantId),
    #[error("unexpected number of {0} actions enqueued for component {1}")]
    UnexpectedNumberOfActionsEnqueuedForComponent(ActionKind, ComponentId),
    #[error("Workspace Snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<ActionPrototypeError> for ActionError {
    fn from(value: ActionPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<crate::attribute::prototype::AttributePrototypeError> for ActionError {
    fn from(value: crate::attribute::prototype::AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<crate::attribute::prototype::argument::AttributePrototypeArgumentError> for ActionError {
    fn from(value: crate::attribute::prototype::argument::AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for ActionError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for ActionError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for ActionError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<DependentValueRootError> for ActionError {
    fn from(value: DependentValueRootError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for ActionError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<HelperError> for ActionError {
    fn from(value: HelperError) -> Self {
        Box::new(value).into()
    }
}

impl From<InferredConnectionGraphError> for ActionError {
    fn from(value: InferredConnectionGraphError) -> Self {
        Box::new(value).into()
    }
}

impl From<NodeWeightError> for ActionError {
    fn from(value: NodeWeightError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for ActionError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for ActionError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for ActionError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

pub type ActionResult<T> = Result<T, ActionError>;

pub use si_events::ActionState;
pub use si_id::{
    ActionId,
    ActionPrototypeId,
};

/// The completion status of a [`ActionRunner`]
///
/// NOTE: This type is only here for backwards comppatibility
/// TODO(fnichol): delete this when it's time
#[remain::sorted]
#[derive(
    Deserialize,
    Serialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ToSql,
    FromSql,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ActionCompletionStatus {
    Error,
    Failure,
    Success,
    Unstarted,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Action {
    id: ActionId,
    state: ActionState,
    originating_changeset_id: ChangeSetId,
    // DEPRECATED
    func_execution_pk: Option<FuncExecutionPk>,
}

impl From<ActionNodeWeight> for Action {
    fn from(value: ActionNodeWeight) -> Self {
        Self {
            id: value.id().into(),
            state: value.state(),
            originating_changeset_id: value.originating_change_set_id(),
            func_execution_pk: None,
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

    /// Returns whether or not any Actions were dispatched.
    pub async fn dispatch_actions(ctx: &DalContext) -> ActionResult<bool> {
        let span = current_span_for_instrument_at!("info");
        let mut actions_dispatched = 0;
        let mut did_dispatch = false;
        // get a count of actions currently running/dispatched
        for dispatchable_ation_id in Action::eligible_to_dispatch(ctx).await? {
            // only dispatch new ones if there's capacity aka - total parallel actions for a workspace can't exceed 40
            Action::dispatch_action(ctx, dispatchable_ation_id).await?;
            did_dispatch = true;
            actions_dispatched += 1;
        }
        span.record("si.rebase.actions_dispatched", actions_dispatched);
        Ok(did_dispatch)
    }

    pub async fn find_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ActionResult<Vec<ActionId>> {
        let mut actions = vec![];
        let snap = ctx.workspace_snapshot()?;
        let action_category_id = snap
            .get_category_node_or_err(CategoryNodeKind::Action)
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

            if Self::component_id(ctx, action_id).await? == Some(component_id) {
                actions.push(action_id);
            }
        }
        Ok(actions)
    }

    #[instrument(level = "info", skip_all)]
    pub async fn remove_all_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ActionResult<()> {
        let actions_to_remove = Self::find_for_component_id(ctx, component_id).await?;
        for action_id in actions_to_remove {
            Self::remove_by_id(ctx, action_id).await?;
        }
        Ok(())
    }

    pub async fn find_for_states_and_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
        action_states: Vec<ActionState>,
    ) -> ActionResult<Vec<ActionId>> {
        let mut actions = vec![];
        let actions_for_component = Self::find_for_component_id(ctx, component_id).await?;
        for action_id in actions_for_component {
            let action = Self::get_by_id(ctx, action_id).await?;
            if action_states.contains(&action.state()) {
                actions.push(action_id);
            }
        }
        Ok(actions)
    }

    pub async fn find_for_kind_and_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
        action_kind: ActionKind,
    ) -> ActionResult<Vec<ActionId>> {
        let actions_for_component = Self::find_for_component_id(ctx, component_id).await?;
        let mut actions = vec![];
        for action_id in actions_for_component {
            let action_prototype_id = Self::prototype_id(ctx, action_id).await?;
            let action_prototype = ActionPrototype::get_by_id(ctx, action_prototype_id).await?;
            if action_prototype.kind == action_kind {
                actions.push(action_id);
            }
        }
        Ok(actions)
    }

    pub async fn find_equivalent(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        maybe_component_id: Option<ComponentId>,
    ) -> ActionResult<Option<ActionId>> {
        let snap = ctx.workspace_snapshot()?;
        let action_category_id = snap
            .get_category_node_or_err(CategoryNodeKind::Action)
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
            .get_node_weight(id)
            .await?
            .get_action_node_weight()?
            .into();
        Ok(action)
    }

    #[instrument(level = "info", skip_all, fields(si.action.id = ?id, si.action.state = ?state))]
    pub async fn set_state(ctx: &DalContext, id: ActionId, state: ActionState) -> ActionResult<()> {
        let node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight(id)
            .await?
            .get_action_node_weight()?;
        let mut new_node_weight = node_weight.clone();
        new_node_weight.set_state(state);
        ctx.workspace_snapshot()?
            .add_or_replace_node(NodeWeight::Action(new_node_weight))
            .await?;
        Ok(())
    }

    #[deprecated(note = "no longer tracking this")]
    pub async fn set_func_execution_pk(
        _ctx: &DalContext,
        _id: ActionId,
        _pk: Option<FuncExecutionPk>,
    ) -> ActionResult<()> {
        unimplemented!("You should never be setting func_execution_pk; bug!");
    }

    /// Enqueues a new action and publishes a WSEvent
    #[instrument(level = "info", skip(ctx))]
    pub async fn new(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        maybe_component_id: Option<ComponentId>,
    ) -> ActionResult<Self> {
        let new_id: ActionId = ctx.workspace_snapshot()?.generate_ulid().await?.into();
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;

        let originating_change_set_id = ctx.change_set_id();
        let node_weight =
            NodeWeight::new_action(originating_change_set_id, new_id.into(), lineage_id);
        ctx.workspace_snapshot()?
            .add_or_replace_node(node_weight)
            .await?;

        let action_category_id = ctx
            .workspace_snapshot()?
            .get_category_node_or_err(CategoryNodeKind::Action)
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
            .get_node_weight(new_id)
            .await?
            .get_action_node_weight()?
            .into();
        WsEvent::action_list_updated(ctx)
            .await?
            .publish_on_commit(ctx)
            .await?;
        Ok(new_action)
    }

    pub async fn remove_by_id(ctx: &DalContext, action_id: ActionId) -> ActionResult<()> {
        ctx.workspace_snapshot()?
            .remove_node_by_id(action_id)
            .await?;
        Ok(())
    }

    pub async fn remove_by_prototype_and_component(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        maybe_component_id: Option<ComponentId>,
    ) -> ActionResult<()> {
        let snap = ctx.workspace_snapshot()?;

        if let Some(action_id) =
            Self::find_equivalent(ctx, action_prototype_id, maybe_component_id).await?
        {
            snap.remove_node_by_id(action_id).await?;
        }
        Ok(())
    }

    /// Sort the dependency graph of [`Actions`][Action] topologically, breaking ties by listing
    /// [`Actions`][Action] sorted by their ID (oldest first thanks to ULID sorting).
    #[instrument(level = "debug", skip_all)]
    pub async fn list_topologically(ctx: &DalContext) -> ActionResult<Vec<ActionId>> {
        let mut action_dependency_graph = ActionDependencyGraph::for_workspace(ctx).await?;
        let mut result = Vec::with_capacity(action_dependency_graph.remaining_actions().len());

        // TODO: Grab all "running" & "failed" Actions to list first?
        loop {
            let mut independent_actions = action_dependency_graph.independent_actions();
            if independent_actions.is_empty() {
                break;
            }

            independent_actions.sort();
            result.reserve(independent_actions.len());
            for action_id in independent_actions {
                action_dependency_graph.remove_action(action_id);
                result.push(action_id);
            }
        }

        // If there is a cycle in the dependencies for some reason, we still want to know that the
        // actions exist, even though they'll never start executing.
        let mut actions_in_cycle = action_dependency_graph.remaining_actions();
        actions_in_cycle.sort();
        result.append(&mut actions_in_cycle);

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

    pub async fn prototype(ctx: &DalContext, action_id: ActionId) -> ActionResult<ActionPrototype> {
        let prototype_id = Self::prototype_id(ctx, action_id).await?;
        Ok(ActionPrototype::get_by_id(ctx, prototype_id).await?)
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

    pub async fn component(
        ctx: &DalContext,
        action_id: ActionId,
    ) -> ActionResult<Option<Component>> {
        match Self::component_id(ctx, action_id).await? {
            Some(component_id) => Ok(Some(Component::get_by_id(ctx, component_id).await?)),
            None => Ok(None),
        }
    }

    pub async fn all_ids(ctx: &DalContext) -> ActionResult<Vec<ActionId>> {
        let action_category_node_index = ctx
            .workspace_snapshot()?
            .get_category_node_or_err(CategoryNodeKind::Action)
            .await?;
        let action_edges = ctx
            .workspace_snapshot()?
            .edges_directed(action_category_node_index, Outgoing)
            .await?;
        let mut result = Vec::with_capacity(action_edges.len());
        for (_, _, action_node_index) in action_edges {
            let action_id = ctx
                .workspace_snapshot()?
                .get_node_weight(action_node_index)
                .await?
                .id();
            result.push(action_id.into());
        }

        Ok(result)
    }

    /// For a given [ActionId] and [ActionDependencyGraph], determine which (if any) actions
    /// are influencing the current action's hold status.
    /// For example, the given Action might be queued, but if it's dependent on an action that is failed or on
    /// hold, we will not dispatch the action
    #[instrument(
        name = "action.get_hold_status_influenced_by",
        level = "debug",
        skip(ctx, action_dependency_graph)
    )]
    pub async fn get_hold_status_influenced_by(
        ctx: &DalContext,
        action_dependency_graph: &ActionDependencyGraph,
        for_action_id: ActionId,
    ) -> ActionResult<Vec<ActionId>> {
        let mut reasons_for_hold = vec![];

        let mut seen_list = HashSet::new();

        let mut work_queue =
            VecDeque::from(action_dependency_graph.direct_dependencies_of(for_action_id));
        while let Some(action_id) = work_queue.pop_front() {
            let act = Self::get_by_id(ctx, action_id).await?;
            match act.state() {
                ActionState::Failed | ActionState::OnHold => reasons_for_hold.push(act.id()),
                _ => (),
            }
            seen_list.insert(action_id);
            for direct_dependency in action_dependency_graph.direct_dependencies_of(action_id) {
                if !seen_list.contains(&direct_dependency) {
                    work_queue.push_back(direct_dependency);
                }
            }
        }
        Ok(reasons_for_hold)
    }

    /// An Action is dispatchable if all of the following are true:
    ///   * The action is in the state [`ActionState::Queued`](ActionState)
    ///   * The graph of values for `DependentValuesUpdate` does *NOT* include
    ///     *ANY* [`AttributeValue`s](AttributeValue) for the same
    ///     [`Component`](crate::Component) as the [`Action`].
    ///
    /// This method **DOES NOT** check the `DependentValuesUpdate` graph. That
    /// is done as part of [`Self::eligible_to_dispatch()`]
    pub fn is_eligible_to_dispatch(&self) -> bool {
        // Only Actions in the ActionState::Queued state are dispatchable.
        self.state() == ActionState::Queued
    }

    /// An Action is dispatchable if all of the following are true:
    ///   * The action is in the state [`ActionState::Queued`](ActionState)
    ///   * The graph of values for `DependentValuesUpdate` does *NOT* include
    ///     *ANY* [`AttributeValue`s](AttributeValue) for the same
    ///     [`Component`](crate::Component) as the [`Action`].
    pub async fn eligible_to_dispatch(ctx: &DalContext) -> ActionResult<Vec<ActionId>> {
        let span = current_span_for_instrument_at!("info");
        let start = Instant::now();
        let action_dependency_graph = ActionDependencyGraph::for_workspace(ctx).await?;
        span.record(
            "si.rebase.action_dependency_graph_time",
            start.elapsed().as_millis(),
        );
        let mut result = Vec::with_capacity(action_dependency_graph.remaining_actions().len());
        let dependent_value_graph = DependentValueGraph::new(
            ctx,
            DependentValueRoot::get_dependent_value_roots(ctx).await?,
        )
        .await?;
        span.record(
            "si.rebase.dependent_value_graph_time",
            start.elapsed().as_millis(),
        );

        // Find the ComponentIds for all AttributeValues in the full dependency graph for the
        // queued/running DependentValuesUpdate. We'll want to hold off on dispatching any Actions
        // that would be operating on the same Component until after the DependentValuesUpdate has
        // finished working with that Component.
        let mut dvu_component_ids: HashSet<ComponentId> = HashSet::new();
        for av_id in &dependent_value_graph.all_value_ids() {
            dvu_component_ids.insert(AttributeValue::component_id(ctx, *av_id).await?);
        }

        for possible_action_id in action_dependency_graph.independent_actions() {
            let action = Action::get_by_id(ctx, possible_action_id).await?;

            if action.is_eligible_to_dispatch() {
                if let Some(action_component_id) = Action::component_id(ctx, action.id()).await? {
                    if dvu_component_ids.contains(&action_component_id) {
                        // This action is for a Component that currently involved in the queued
                        // DependentValuesUpdate graph. We don't want to dispatch the Action until
                        // the DependentValuesUpdate job has completely finished
                        // processing/populating values that the Action might need to work with.
                        continue;
                    }
                }
                result.push(possible_action_id);
            }
        }

        Ok(result)
    }

    #[instrument(name = "workspace_snapshot.dispatch_action", level = "info", skip_all, fields(
        si.action.id = ?action_id,
    ))]
    pub async fn dispatch_action(ctx: &DalContext, action_id: ActionId) -> ActionResult<()> {
        Action::set_state(ctx, action_id, ActionState::Dispatched).await?;

        ctx.enqueue_action_job(ctx.workspace_pk()?, ctx.change_set_id(), action_id)
            .await?;

        Ok(())
    }

    pub async fn last_func_run_id_for_id_opt(
        ctx: &DalContext,
        id: ActionId,
    ) -> ActionResult<Option<FuncRunId>> {
        Ok(ctx
            .layer_db()
            .func_run()
            .get_last_run_for_action_id_opt(ctx.events_tenancy().workspace_pk, id)
            .await?
            .map(|f| f.id()))
    }

    /// This function behaves differently if on head vs. in an open change set.
    /// For a given component, if we're on head already, we'll simply enqueue the refresh action as normal
    /// If we're not on head, we check to see if the component exists on head
    /// If it does, we enqueue the refresh function on head (and when it runs, it will be replayed)
    /// as we do not want a change set to have a more up-to-date perspective than head
    /// If it does NOT exist on head, we will dispatch the refresh action directly!
    pub async fn enqueue_refresh_in_correct_change_set_and_commit(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ActionResult<()> {
        let head_change_set_id = ctx.get_workspace_default_change_set_id().await?;
        // if we're already on head, just enqueue it
        if ctx.change_set_id() == head_change_set_id {
            Self::enqueue_refresh(ctx, component_id, false).await
        } else {
            // Not on head, so build the head dal ctx
            let head_ctx = ctx.clone_with_head().await?;
            // if the component exists on head, enqueue the refresh action there
            if Component::exists_by_id(&head_ctx, component_id).await? {
                Self::enqueue_refresh(&head_ctx, component_id, false).await
            }
            // otherwise, if the component doesn't have a resource, just enqueue it
            // (this is a backend guard, the button will be hidden from the user)
            else if Component::resource_by_id(ctx, component_id)
                .await?
                .is_none()
            {
                Self::enqueue_refresh(ctx, component_id, false).await
            }
            // last case, component doesn't exist on head, but has a resource (which can only be true if we ran an import or other mgmt func)
            // so we can enqueue and dispatch right away
            else {
                Self::enqueue_refresh(ctx, component_id, true).await
            }
        }
    }

    async fn enqueue_refresh(
        ctx: &DalContext,
        component_id: ComponentId,
        should_dispatch: bool,
    ) -> ActionResult<()> {
        let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
        let refresh_actions = SchemaVariant::find_action_prototypes_by_kind(
            ctx,
            schema_variant_id,
            ActionKind::Refresh,
        )
        .await?;
        if let Ok(&prototype_id) = refresh_actions.iter().exactly_one().map_err(|_| {
            ActionError::UnexpectedNumberOfActionKinds(ActionKind::Refresh, schema_variant_id)
        }) {
            let maybe_duplicate_action =
                Action::find_for_kind_and_component_id(ctx, component_id, ActionKind::Refresh)
                    .await?;

            // See if there's an existing Refresh Action (single) for this component
            if let Some(&action_id) = maybe_duplicate_action.iter().at_most_one().map_err(|_| {
                ActionError::UnexpectedNumberOfActionsEnqueuedForComponent(
                    ActionKind::Refresh,
                    component_id,
                )
            })? {
                if should_dispatch {
                    // If we're dispatching and there's already an action enqueued, and the originating change set
                    // is this change set, dispatch it! Otherwise, create a new action and dispatch it.
                    let action = Action::get_by_id(ctx, action_id).await?;
                    if action.originating_changeset_id() == ctx.change_set_id() {
                        Action::dispatch_action(ctx, action_id).await?;
                    } else {
                        let new_action_id =
                            Self::enqueue_new_refresh(ctx, prototype_id, component_id).await?;
                        Action::dispatch_action(ctx, new_action_id).await?;
                    }
                } else {
                    // Not dispatching - re-enqueue the existing action
                    let action = Action::get_by_id(ctx, action_id).await?;
                    match action.state() {
                        ActionState::Failed | ActionState::OnHold => {
                            Action::set_state(ctx, action_id, ActionState::Queued).await?;
                        }
                        ActionState::Dispatched | ActionState::Queued | ActionState::Running => {
                            // no op if the action is already queued/dispatched/running
                            return Ok(());
                        }
                    }
                }
            } else {
                // No duplicate actions - create a new one and optionally dispatch
                let action_id = Self::enqueue_new_refresh(ctx, prototype_id, component_id).await?;
                if should_dispatch {
                    Action::dispatch_action(ctx, action_id).await?;
                }
            }
            ctx.commit().await?;
        };

        Ok(())
    }

    async fn enqueue_new_refresh(
        ctx: &DalContext,
        prototype_id: ActionPrototypeId,
        component_id: ComponentId,
    ) -> ActionResult<ActionId> {
        let func_id = ActionPrototype::func_id(ctx, prototype_id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;
        let action = Action::new(ctx, prototype_id, Some(component_id)).await?;
        ctx.write_audit_log(
            AuditLogKind::AddAction {
                prototype_id,
                action_kind: si_events::ActionKind::Refresh,
                func_id,
                func_display_name: func.display_name,
                func_name: func.name.clone(),
            },
            func.name,
        )
        .await?;
        Ok(action.id())
    }
}

impl WsEvent {
    pub async fn action_list_updated(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ActionsListUpdated(ctx.change_set_id())).await
    }
}
