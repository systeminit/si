use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use si_pkg::ActionFuncSpecKind;
use std::collections::HashMap;
use std::sync::Arc;
use strum::{AsRefStr, Display};
use thiserror::Error;
use veritech_client::OutputStream;

use crate::attribute::prototype::AttributePrototypeResult;
use crate::change_set::ChangeSetError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use crate::workspace_snapshot::edge_weight::{EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{
    NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::SchemaVariant;
use crate::{
    func::backend::js_action::DeprecatedActionRunResult,
    func::binding::return_value::FuncBindingReturnValueError,
    func::binding::{FuncBinding, FuncBindingError},
    implement_add_edge_to,
    layer_db_types::{DeprecatedActionPrototypeContent, DeprecatedActionPrototypeContentV1},
    secret::{before_funcs_for_component, BeforeFuncError},
    ActionPrototypeId, Component, ComponentError, ComponentId, DalContext, FuncError, FuncId,
    HelperError, SchemaVariantError, SchemaVariantId, Timestamp, TransactionsError, WsEvent,
    WsEventError, WsEventResult, WsPayload,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DeprecatedActionPrototypeError {
    #[error(transparent)]
    BeforeFunc(#[from] BeforeFuncError),
    #[error(transparent)]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("action prototype {0} is missing a function edge")]
    MissingFunction(ActionPrototypeId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type DeprecatedActionPrototypeResult<T> = Result<T, DeprecatedActionPrototypeError>;

/// Describes how an [`Action`](ActionPrototype) affects the world.
#[remain::sorted]
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DeprecatedActionKind {
    /// The [`action`](ActionPrototype) creates a new "resource".
    Create,
    /// The [`action`](ActionPrototype) deletes an existing "resource".
    Delete,
    /// The [`action`](ActionPrototype) is "internal only" or has multiple effects.
    Other,
    /// The [`action`](ActionPrototype) that refreshes an existing "resource".
    Refresh,
    /// Unused - added so we can create actions v2 forward compatible schemas
    Update,
}

impl From<ActionFuncSpecKind> for DeprecatedActionKind {
    fn from(value: ActionFuncSpecKind) -> Self {
        match value {
            ActionFuncSpecKind::Create => DeprecatedActionKind::Create,
            ActionFuncSpecKind::Refresh => DeprecatedActionKind::Refresh,
            ActionFuncSpecKind::Other => DeprecatedActionKind::Other,
            ActionFuncSpecKind::Delete => DeprecatedActionKind::Delete,
            ActionFuncSpecKind::Update => DeprecatedActionKind::Other,
        }
    }
}

impl From<&DeprecatedActionKind> for ActionFuncSpecKind {
    fn from(value: &DeprecatedActionKind) -> Self {
        match value {
            DeprecatedActionKind::Create => ActionFuncSpecKind::Create,
            DeprecatedActionKind::Refresh => ActionFuncSpecKind::Refresh,
            DeprecatedActionKind::Other => ActionFuncSpecKind::Other,
            DeprecatedActionKind::Delete => ActionFuncSpecKind::Delete,
            DeprecatedActionKind::Update => ActionFuncSpecKind::Update,
        }
    }
}

// An ActionPrototype joins a `FuncId` to a `SchemaVariantId` with a `ActionKind` and `name`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedActionPrototype {
    pub id: ActionPrototypeId,
    pub kind: DeprecatedActionKind,
    pub name: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl DeprecatedActionPrototype {
    pub fn assemble(id: ActionPrototypeId, content: DeprecatedActionPrototypeContentV1) -> Self {
        Self {
            id,
            name: content.name,
            kind: content.kind,
            timestamp: content.timestamp,
        }
    }

    implement_add_edge_to!(
        source_id: ActionPrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: DeprecatedActionPrototypeResult,
    );

    pub async fn new(
        ctx: &DalContext,
        name: Option<impl Into<String>>,
        kind: DeprecatedActionKind,
        schema_variant_id: SchemaVariantId,
        func_id: FuncId,
    ) -> DeprecatedActionPrototypeResult<Self> {
        let timestamp = Timestamp::now();

        let content = DeprecatedActionPrototypeContentV1 {
            kind,
            timestamp,
            name: name.map(Into::into),
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(DeprecatedActionPrototypeContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ActionPrototype(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot.add_node(node_weight.to_owned()).await?;
        SchemaVariant::add_edge_to_deprecated_action_prototype(
            ctx,
            schema_variant_id,
            id.into(),
            EdgeWeightKind::ActionPrototype,
        )
        .await?;

        let prototype = DeprecatedActionPrototype::assemble(id.into(), content);
        DeprecatedActionPrototype::add_edge_to_func(
            ctx,
            prototype.id,
            func_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        Ok(prototype)
    }

    pub async fn for_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> DeprecatedActionPrototypeResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let nodes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::ActionPrototype,
            )
            .await?;
        let mut node_weights = Vec::with_capacity(nodes.len());
        let mut content_hashes = Vec::with_capacity(nodes.len());
        for node in nodes {
            let weight = workspace_snapshot.get_node_weight(node).await?;
            if let NodeWeight::Content(content) = &weight {
                if matches!(
                    content.content_address(),
                    ContentAddress::ActionPrototype(_)
                ) {
                    content_hashes.push(weight.content_hash());
                    node_weights.push(weight);
                }
            }
        }

        let content_map: HashMap<ContentHash, DeprecatedActionPrototypeContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
            .await?;

        let mut prototypes = Vec::with_capacity(node_weights.len());
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let DeprecatedActionPrototypeContent::V1(inner) = content;

                    prototypes.push(Self::assemble(node_weight.id().into(), inner.clone()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }
        Ok(prototypes)
    }

    pub async fn get_by_id_or_error(
        ctx: &DalContext,
        id: ActionPrototypeId,
    ) -> DeprecatedActionPrototypeResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let ulid: ::si_events::ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: DeprecatedActionPrototypeContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let DeprecatedActionPrototypeContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    /// Lists all [`DeprecatedActionPrototypes`](DeprecatedActionPrototype) for a given
    /// [`FuncId`](Func).
    pub async fn list_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> DeprecatedActionPrototypeResult<Vec<ActionPrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut action_prototype_ids = Vec::new();
        for node_index in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            if let Some(ContentAddressDiscriminants::ActionPrototype) =
                node_weight.content_address_discriminants()
            {
                action_prototype_ids.push(node_weight.id().into());
            }
        }

        Ok(action_prototype_ids)
    }

    pub async fn func_id_by_id(
        ctx: &DalContext,
        id: ActionPrototypeId,
    ) -> DeprecatedActionPrototypeResult<FuncId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let id = node_weight.id();
            if NodeWeightDiscriminants::Func == node_weight.into() {
                return Ok(id.into());
            }
        }

        Err(DeprecatedActionPrototypeError::MissingFunction(id))
    }

    pub async fn func_id(&self, ctx: &DalContext) -> DeprecatedActionPrototypeResult<FuncId> {
        Self::func_id_by_id(ctx, self.id).await
    }

    pub async fn run(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> DeprecatedActionPrototypeResult<(Option<DeprecatedActionRunResult>, Vec<OutputStream>)>
    {
        let component = Component::get_by_id(ctx, component_id).await?;
        let component_view = component.view(ctx).await?;

        let before = before_funcs_for_component(ctx, component_id).await?;

        let (_, return_value) = FuncBinding::create_and_execute(
            ctx,
            serde_json::json!({ "properties" : component_view }),
            self.func_id(ctx).await?,
            before,
        )
        .await?;

        let mut logs = vec![];
        for stream_part in return_value
            .get_output_stream(ctx)
            .await?
            .unwrap_or_default()
        {
            logs.push(stream_part);
        }

        logs.sort_by_key(|log| log.timestamp);

        Ok((
            match return_value.value() {
                Some(value) => {
                    let run_result: DeprecatedActionRunResult =
                        serde_json::from_value(value.clone())?;

                    component.set_resource(ctx, run_result.clone()).await?;

                    WsEvent::resource_refreshed(ctx, component.id())
                        .await?
                        .publish_on_commit(ctx)
                        .await?;

                    if component.to_delete() && run_result.payload.is_none() {
                        Component::remove(ctx, component.id()).await?;
                    }

                    Some(run_result)
                }
                None => None,
            },
            logs,
        ))
    }

    pub async fn remove(ctx: &DalContext, id: ActionPrototypeId) -> AttributePrototypeResult<()> {
        let change_set = ctx.change_set()?;

        ctx.workspace_snapshot()?
            .remove_node_by_id(change_set, id)
            .await?;

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRefreshedPayload {
    component_id: ComponentId,
}

impl WsEvent {
    pub async fn resource_refreshed(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ResourceRefreshed(ResourceRefreshedPayload { component_id }),
        )
        .await
    }
}
