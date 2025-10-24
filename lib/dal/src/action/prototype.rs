use std::sync::Arc;

use petgraph::{
    Direction::Incoming,
    Outgoing,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ActionResultState,
    FuncRunId,
};
use si_frontend_types::DiagramComponentView;
use si_id::{
    SchemaId,
    ulid::Ulid,
};
use si_layer_cache::LayerDbError;
use si_pkg::ActionFuncSpecKind;
use strum::Display;
use thiserror::Error;
use veritech_client::{
    ActionRunResultSuccess,
    ResourceStatus,
};

use super::ActionError;
use crate::{
    ActionPrototypeId,
    ChangeSetError,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    FuncError,
    HelperError,
    Schema,
    SchemaError,
    SchemaVariant,
    SchemaVariantError,
    SchemaVariantId,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    WsEventResult,
    WsPayload,
    action::{
        Action,
        ActionId,
    },
    component::ComponentUpdatedPayload,
    diagram::DiagramError,
    func::{
        FuncId,
        binding::EventualParent,
        runner::{
            FuncRunner,
            FuncRunnerError,
        },
    },
    implement_add_edge_to,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{
            ActionPrototypeNodeWeight,
            NodeWeight,
            NodeWeightDiscriminants,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
        },
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionPrototypeError {
    #[error("action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("diagram error: {0}")]
    Diagram(#[from] Box<DiagramError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func not found for prototype: {0}")]
    FuncNotFoundForPrototype(ActionPrototypeId),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] Box<FuncRunnerError>),
    #[error("func runner has failed to send a value and exited")]
    FuncRunnerSend,
    #[error("Helper error: {0}")]
    Helper(#[from] Box<HelperError>),
    #[error("Layer DB Error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("Node Weight error: {0}")]
    NodeWeight(#[from] Box<NodeWeightError>),
    #[error("Action prototype {0} is orphaned")]
    Orphaned(ActionPrototypeId),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("schema variant not found for prototype: {0}")]
    SchemaVariantNotFoundForPrototype(ActionPrototypeId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("Workspace Snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<ChangeSetError> for ActionPrototypeError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for ActionPrototypeError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<DiagramError> for ActionPrototypeError {
    fn from(value: DiagramError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for ActionPrototypeError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncRunnerError> for ActionPrototypeError {
    fn from(value: FuncRunnerError) -> Self {
        Box::new(value).into()
    }
}

impl From<HelperError> for ActionPrototypeError {
    fn from(value: HelperError) -> Self {
        Box::new(value).into()
    }
}

impl From<NodeWeightError> for ActionPrototypeError {
    fn from(value: NodeWeightError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for ActionPrototypeError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for ActionPrototypeError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for ActionPrototypeError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for ActionPrototypeError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

pub type ActionPrototypeResult<T> = Result<T, ActionPrototypeError>;

#[remain::sorted]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Display, Hash)]
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

impl From<ActionKind> for si_events::ActionKind {
    fn from(value: ActionKind) -> Self {
        match value {
            ActionKind::Create => si_events::ActionKind::Create,
            ActionKind::Destroy => si_events::ActionKind::Destroy,
            ActionKind::Manual => si_events::ActionKind::Manual,
            ActionKind::Refresh => si_events::ActionKind::Refresh,
            ActionKind::Update => si_events::ActionKind::Update,
        }
    }
}

impl From<si_events::ActionKind> for ActionKind {
    fn from(value: si_events::ActionKind) -> Self {
        match value {
            si_events::ActionKind::Create => ActionKind::Create,
            si_events::ActionKind::Destroy => ActionKind::Destroy,
            si_events::ActionKind::Manual => ActionKind::Manual,
            si_events::ActionKind::Refresh => ActionKind::Refresh,
            si_events::ActionKind::Update => ActionKind::Update,
        }
    }
}

impl From<ActionFuncSpecKind> for ActionKind {
    fn from(value: ActionFuncSpecKind) -> Self {
        match value {
            ActionFuncSpecKind::Create => ActionKind::Create,
            ActionFuncSpecKind::Refresh => ActionKind::Refresh,
            ActionFuncSpecKind::Other => ActionKind::Manual,
            ActionFuncSpecKind::Delete => ActionKind::Destroy,
            ActionFuncSpecKind::Update => ActionKind::Update,
        }
    }
}

impl From<ActionKind> for ActionFuncSpecKind {
    fn from(value: ActionKind) -> Self {
        match value {
            ActionKind::Create => ActionFuncSpecKind::Create,
            ActionKind::Destroy => ActionFuncSpecKind::Delete,
            ActionKind::Manual => ActionFuncSpecKind::Other,
            ActionKind::Refresh => ActionFuncSpecKind::Refresh,
            ActionKind::Update => ActionFuncSpecKind::Update,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActionPrototypeParent {
    Schemas(Vec<SchemaId>),
    SchemaVariant(SchemaVariantId),
}

impl From<ActionPrototypeParent> for EventualParent {
    fn from(value: ActionPrototypeParent) -> Self {
        match value {
            ActionPrototypeParent::Schemas(ids) => EventualParent::Schemas(ids),
            ActionPrototypeParent::SchemaVariant(id) => EventualParent::SchemaVariant(id),
        }
    }
}

impl From<SchemaId> for ActionPrototypeParent {
    fn from(value: SchemaId) -> Self {
        ActionPrototypeParent::Schemas(vec![value])
    }
}

impl From<SchemaVariantId> for ActionPrototypeParent {
    fn from(value: SchemaVariantId) -> Self {
        ActionPrototypeParent::SchemaVariant(value)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionPrototype {
    pub id: ActionPrototypeId,
    pub kind: ActionKind,
    pub name: String,
    pub description: Option<String>,
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

    /// Make this action prototype an "overlay" action prototype.
    pub async fn promote_to_overlay(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
    ) -> ActionPrototypeResult<()> {
        let overlay_category_id = ctx
            .workspace_snapshot()?
            .get_or_create_static_category_node(CategoryNodeKind::Overlays)
            .await?;
        Self::add_overlay_category_edge(
            ctx,
            overlay_category_id,
            action_prototype_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        Ok(())
    }

    pub async fn new(
        ctx: &DalContext,
        kind: ActionKind,
        name: String,
        description: Option<String>,
        parent_id: impl Into<ActionPrototypeParent>,
        func_id: FuncId,
    ) -> ActionPrototypeResult<Self> {
        let new_id: ActionPrototypeId = ctx.workspace_snapshot()?.generate_ulid().await?.into();
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let parent_id = parent_id.into();
        let node_weight =
            NodeWeight::new_action_prototype(new_id.into(), lineage_id, kind, name, description);
        ctx.workspace_snapshot()?
            .add_or_replace_node(node_weight)
            .await?;

        Self::add_edge_to_func(ctx, new_id, func_id, EdgeWeightKind::new_use()).await?;

        match parent_id {
            ActionPrototypeParent::Schemas(schema_ids) => {
                for schema_id in schema_ids {
                    Schema::add_edge_to_action_prototype(
                        ctx,
                        schema_id,
                        new_id,
                        EdgeWeightKind::ActionPrototype,
                    )
                    .await
                    .map_err(Box::new)?;
                }

                Self::promote_to_overlay(ctx, new_id).await?;
            }
            ActionPrototypeParent::SchemaVariant(schema_variant_id) => {
                SchemaVariant::add_edge_to_action_prototype(
                    ctx,
                    schema_variant_id,
                    new_id,
                    EdgeWeightKind::ActionPrototype,
                )
                .await?;
            }
        }

        let new_prototype: Self = ctx
            .workspace_snapshot()?
            .get_node_weight(new_id)
            .await?
            .get_action_prototype_node_weight()?
            .into();

        Ok(new_prototype)
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    implement_add_edge_to!(
        source_id: ActionPrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ActionPrototypeResult,
    );

    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: ActionPrototypeId,
        add_fn: add_overlay_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ActionPrototypeResult,
    );

    pub async fn get_by_id(ctx: &DalContext, id: ActionPrototypeId) -> ActionPrototypeResult<Self> {
        let prototype: Self = ctx
            .workspace_snapshot()?
            .get_node_weight(id)
            .await?
            .get_action_prototype_node_weight()?
            .into();
        Ok(prototype)
    }

    pub async fn func_id(ctx: &DalContext, id: ActionPrototypeId) -> ActionPrototypeResult<FuncId> {
        for (_, _, target_id) in ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(id, Outgoing, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            if let NodeWeight::Func(node_weight) =
                ctx.workspace_snapshot()?.get_node_weight(target_id).await?
            {
                return Ok(node_weight.id().into());
            }
        }

        Err(ActionPrototypeError::FuncNotFoundForPrototype(id))
    }

    /// Lists all [`ActionPrototypes`](ActionPrototype) for a given
    /// [`FuncId`](Func).
    pub async fn list_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> ActionPrototypeResult<Vec<ActionPrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let action_prototype_nodes = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?;
        let mut action_prototype_ids = Vec::with_capacity(action_prototype_nodes.len());
        for node_index in action_prototype_nodes {
            if let NodeWeight::ActionPrototype(node_weight) =
                workspace_snapshot.get_node_weight(node_index).await?
            {
                action_prototype_ids.push(node_weight.id().into());
            }
        }

        Ok(action_prototype_ids)
    }

    pub async fn parentage(
        ctx: &DalContext,
        id: ActionPrototypeId,
    ) -> ActionPrototypeResult<ActionPrototypeParent> {
        if let Some(schema_ids) = Self::schema_ids(ctx, id).await? {
            Ok(ActionPrototypeParent::Schemas(schema_ids))
        } else if let Some(schema_variant_id) = Self::schema_variant_id(ctx, id).await? {
            Ok(ActionPrototypeParent::SchemaVariant(schema_variant_id))
        } else {
            Err(ActionPrototypeError::Orphaned(id))
        }
    }

    /// If this Action Prototype is parented by one or more Schema(s), returns the Schema ids.
    pub async fn schema_ids(
        ctx: &DalContext,
        id: ActionPrototypeId,
    ) -> ActionPrototypeResult<Option<Vec<SchemaId>>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut result = vec![];

        // Schemas have an outgoing edge to the action prototype, which have an incoming edge from the schema
        let maybe_schemas = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::ActionPrototype)
            .await?;

        for maybe_schema_id in maybe_schemas {
            if let NodeWeight::Content(content_weight) = ctx
                .workspace_snapshot()?
                .get_node_weight(maybe_schema_id)
                .await?
            {
                if ContentAddressDiscriminants::Schema == content_weight.content_address().into() {
                    result.push(maybe_schema_id.into());
                }
            }
        }

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// If this Action Prototype is parented by a SchemaVariant, returns the SchemaVariant Id.
    pub async fn schema_variant_id(
        ctx: &DalContext,
        id: ActionPrototypeId,
    ) -> ActionPrototypeResult<Option<SchemaVariantId>> {
        for (_, tail_node_idx, _head_node_idx) in ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                id,
                Incoming,
                EdgeWeightKindDiscriminants::ActionPrototype,
            )
            .await?
        {
            let node_weight = ctx
                .workspace_snapshot()?
                .get_node_weight(tail_node_idx)
                .await?;

            if NodeWeightDiscriminants::from(&node_weight) == NodeWeightDiscriminants::SchemaVariant
            {
                return Ok(Some(node_weight.id().into()));
            } else if let NodeWeight::Content(content_weight) = &node_weight {
                if ContentAddressDiscriminants::from(content_weight.content_address())
                    == ContentAddressDiscriminants::SchemaVariant
                {
                    return Ok(Some(node_weight.id().into()));
                }
            }
        }

        Ok(None)
    }

    /// Find the action prototype for a given kind. If a prototype is one of the
    /// unique ones (`ActionKind::Create`, `ActionKind::Update`,
    /// `ActionKind::Refresh`, `ActionKind::Destroy`), then look first for the
    /// version defined at the schema level, and only look for the variant level
    /// if no schema level prototype is found. For `ActionKind::Manual`, find
    /// prototype at both levels.
    pub async fn find_by_kind_for_schema_or_variant(
        ctx: &DalContext,
        kind: ActionKind,
        schema_variant_id: SchemaVariantId,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let schema_id = SchemaVariant::schema_id(ctx, schema_variant_id).await?;

        let mut prototypes = vec![];
        prototypes.extend(
            Self::for_schema(ctx, schema_id)
                .await?
                .into_iter()
                .filter(|proto| proto.kind == kind),
        );

        if prototypes.is_empty() || kind == ActionKind::Manual {
            prototypes.extend(
                Self::for_variant(ctx, schema_variant_id)
                    .await?
                    .into_iter()
                    .filter(|proto| proto.kind == kind),
            );
        }

        Ok(prototypes)
    }

    /// Find the all action prototypes for a given variant. If a prototype is one of the
    /// unique ones (`ActionKind::Create`, `ActionKind::Update`,
    /// `ActionKind::Refresh`, `ActionKind::Destroy`), then look first for the
    /// version defined at the schema level, and only look for the variant level
    /// if no schema level prototype is found. For `ActionKind::Manual`, find
    /// prototype at both levels.
    /// Return all valid prototypes for the variant
    pub async fn list_for_schema_and_variant_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let mut prototypes = vec![];
        // grab all the different kinds, handling overlays if needed
        let create_actions =
            Self::find_by_kind_for_schema_or_variant(ctx, ActionKind::Create, schema_variant_id)
                .await?;
        prototypes.extend(create_actions);
        let update_actions =
            Self::find_by_kind_for_schema_or_variant(ctx, ActionKind::Update, schema_variant_id)
                .await?;
        prototypes.extend(update_actions);

        let refresh_actions =
            Self::find_by_kind_for_schema_or_variant(ctx, ActionKind::Refresh, schema_variant_id)
                .await?;
        prototypes.extend(refresh_actions);
        let destroy_actions =
            Self::find_by_kind_for_schema_or_variant(ctx, ActionKind::Destroy, schema_variant_id)
                .await?;
        prototypes.extend(destroy_actions);
        let manual_actions =
            Self::find_by_kind_for_schema_or_variant(ctx, ActionKind::Manual, schema_variant_id)
                .await?;
        prototypes.extend(manual_actions);
        Ok(prototypes)
    }

    pub async fn run(
        ctx: &DalContext,
        id: ActionPrototypeId,
        component_id: ComponentId,
    ) -> ActionPrototypeResult<(Option<ActionRunResultSuccess>, FuncRunId)> {
        let component = Component::get_by_id(ctx, component_id).await?;
        let component_view = component.view(ctx).await?;
        let func_id = Self::func_id(ctx, id).await?;

        let result_channel = FuncRunner::run_action(
            ctx,
            id,
            component_id,
            func_id,
            serde_json::json!({ "properties" : component_view }),
        )
        .await?;

        let func_run_value = result_channel
            .await
            .map_err(|_| ActionPrototypeError::FuncRunnerSend)??;

        let content_value: Option<si_events::CasValue> =
            func_run_value.value().cloned().map(Into::into);
        let content_unprocessed_value: Option<si_events::CasValue> =
            func_run_value.unprocessed_value().cloned().map(Into::into);

        let value_address = match content_value {
            Some(value) => Some(
                ctx.layer_db()
                    .cas()
                    .write(
                        Arc::new(value.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )?
                    .0,
            ),
            None => None,
        };

        let unprocessed_value_address = match content_unprocessed_value {
            Some(value) => Some(
                ctx.layer_db()
                    .cas()
                    .write(
                        Arc::new(value.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )?
                    .0,
            ),
            None => None,
        };

        FuncRunner::update_run(ctx, func_run_value.func_run_id(), |func_run| {
            func_run.set_success(unprocessed_value_address, value_address);
        })
        .await?;

        let maybe_run_result = match func_run_value.value() {
            Some(value) => Some(serde_json::from_value::<ActionRunResultSuccess>(
                value.clone(),
            )?),
            None => None,
        };

        match maybe_run_result.as_ref().map(|r| r.status) {
            // If we have a resource and an ok status
            Some(ResourceStatus::Ok) => {
                // Set the `FuncRun`'s action-specific metadata to successful
                FuncRunner::update_run(ctx, func_run_value.func_run_id(), |func_run| {
                    func_run.set_action_result_state(Some(ActionResultState::Success))
                })
                .await?;
            }
            // In all other cases
            Some(_) | None => {
                // Set the `FuncRun`'s action-specific metadata to falure
                FuncRunner::update_run(ctx, func_run_value.func_run_id(), |func_run| {
                    func_run.set_action_result_state(Some(ActionResultState::Failure))
                })
                .await?;
            }
        }

        Ok((maybe_run_result, func_run_value.func_run_id()))
    }

    pub async fn for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let prototype_edges = ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                schema_id,
                Outgoing,
                EdgeWeightKindDiscriminants::ActionPrototype,
            )
            .await?;
        let mut prototypes = Vec::with_capacity(prototype_edges.len());
        for (_, _, head_node_id) in prototype_edges {
            if let NodeWeight::ActionPrototype(node_weight) = ctx
                .workspace_snapshot()?
                .get_node_weight(head_node_id)
                .await?
            {
                prototypes.push(node_weight.into());
            }
        }

        Ok(prototypes)
    }

    pub async fn for_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let prototype_edges = ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                schema_variant_id,
                Outgoing,
                EdgeWeightKindDiscriminants::ActionPrototype,
            )
            .await?;
        let mut prototypes = Vec::with_capacity(prototype_edges.len());
        for (_, _, head_node_id) in prototype_edges {
            if let NodeWeight::ActionPrototype(node_weight) = ctx
                .workspace_snapshot()?
                .get_node_weight(head_node_id)
                .await?
            {
                prototypes.push(node_weight.into());
            }
        }

        Ok(prototypes)
    }

    pub async fn get_prototypes_to_trigger(
        ctx: &DalContext,
        action_id: ActionId,
        prototype_id: ActionPrototypeId,
    ) -> ActionPrototypeResult<Vec<ActionPrototypeId>> {
        // for now we are only defaulting to triggering a
        // refresh when a create action succeeds
        // in the future, this will be configurable and we'll look up edges
        let action_prototype = Self::get_by_id(ctx, prototype_id).await?;

        // Currently we only trigger a refresh for a create
        if action_prototype.kind != ActionKind::Create {
            return Ok(vec![]);
        }

        // Although we may in the future support actions without components,
        // right now that is not meaningful
        let Some(component_id) = Action::component_id(ctx, action_id)
            .await
            .map_err(Box::new)?
        else {
            return Ok(vec![]);
        };

        let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

        Ok(
            Self::find_by_kind_for_schema_or_variant(ctx, ActionKind::Refresh, schema_variant_id)
                .await?
                .into_iter()
                .map(|proto| proto.id())
                .collect(),
        )
    }

    async fn find_enqueued_actions(
        ctx: &DalContext,
        id: ActionPrototypeId,
    ) -> ActionPrototypeResult<Vec<ActionId>> {
        let mut enqueued_actions = vec![];

        for (_, tail_node_idx, _head_node_idx) in ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(id, Incoming, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            if let NodeWeight::Action(node_weight) = ctx
                .workspace_snapshot()?
                .get_node_weight(tail_node_idx)
                .await?
            {
                enqueued_actions.push(node_weight.id().into());
            }
        }
        Ok(enqueued_actions)
    }

    pub async fn remove(ctx: &DalContext, id: ActionPrototypeId) -> ActionPrototypeResult<()> {
        // check if there are existing actions queued for this prototype and remove them
        let enqueued_actions = Self::find_enqueued_actions(ctx, id).await?;

        for action in enqueued_actions {
            ctx.workspace_snapshot()?.remove_node_by_id(action).await?;
            WsEvent::action_list_updated(ctx)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
        ctx.workspace_snapshot()?.remove_node_by_id(id).await?;

        Ok(())
    }
}

impl WsEvent {
    pub async fn resource_refreshed(
        ctx: &DalContext,
        payload: DiagramComponentView,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ResourceRefreshed(ComponentUpdatedPayload {
                component: payload,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }
}
