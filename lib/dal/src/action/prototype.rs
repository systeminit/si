use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use si_pkg::ActionFuncSpecKind;
use std::collections::HashMap;
use std::sync::Arc;
use strum::{AsRefStr, Display};
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{
    NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    func::backend::js_action::ActionRunResult,
    func::before::{before_funcs_for_component, BeforeFuncError},
    func::binding::{FuncBinding, FuncBindingError},
    func::binding_return_value::FuncBindingReturnValueError,
    layer_db_types::{ActionPrototypeContent, ActionPrototypeContentV1},
    pk, Component, ComponentError, ComponentId, DalContext, Func, FuncError, FuncId,
    SchemaVariantError, SchemaVariantId, Timestamp, TransactionsError, WsEvent, WsEventError,
    WsEventResult, WsPayload,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionPrototypeView {
    id: ActionPrototypeId,
    name: String,
    display_name: Option<String>,
}

impl ActionPrototypeView {
    pub async fn new(
        ctx: &DalContext,
        prototype: ActionPrototype,
    ) -> ActionPrototypeResult<ActionPrototypeView> {
        let func = Func::get_by_id(ctx, prototype.func_id(ctx).await?).await?;
        let display_name = func.display_name.map(|dname| dname.to_string());
        Ok(Self {
            id: prototype.id,
            name: prototype.name.as_deref().map_or_else(
                || match prototype.kind {
                    ActionKind::Create => "create".to_owned(),
                    ActionKind::Delete => "delete".to_owned(),
                    ActionKind::Other => "other".to_owned(),
                    ActionKind::Refresh => "refresh".to_owned(),
                },
                ToOwned::to_owned,
            ),
            display_name,
        })
    }
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionPrototypeError {
    #[error(transparent)]
    BeforeFunc(#[from] BeforeFuncError),
    #[error(transparent)]
    ChangeSetPointer(#[from] ChangeSetPointerError),
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

pub type ActionPrototypeResult<T> = Result<T, ActionPrototypeError>;

/// Describes how an [`Action`](ActionPrototype) affects the world.
#[remain::sorted]
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ActionKind {
    /// The [`action`](ActionPrototype) creates a new "resource".
    Create,
    /// The [`action`](ActionPrototype) deletes an existing "resource".
    Delete,
    /// The [`action`](ActionPrototype) is "internal only" or has multiple effects.
    Other,
    /// The [`action`](ActionPrototype) that refreshes an existing "resource".
    Refresh,
}

impl From<ActionFuncSpecKind> for ActionKind {
    fn from(value: ActionFuncSpecKind) -> Self {
        match value {
            ActionFuncSpecKind::Create => ActionKind::Create,
            ActionFuncSpecKind::Refresh => ActionKind::Refresh,
            ActionFuncSpecKind::Other => ActionKind::Other,
            ActionFuncSpecKind::Delete => ActionKind::Delete,
        }
    }
}

impl From<&ActionKind> for ActionFuncSpecKind {
    fn from(value: &ActionKind) -> Self {
        match value {
            ActionKind::Create => ActionFuncSpecKind::Create,
            ActionKind::Refresh => ActionFuncSpecKind::Refresh,
            ActionKind::Other => ActionFuncSpecKind::Other,
            ActionKind::Delete => ActionFuncSpecKind::Delete,
        }
    }
}

pk!(ActionPrototypeId);

// An ActionPrototype joins a `FuncId` to a `SchemaVariantId` with a `ActionKind` and `name`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionPrototype {
    pub id: ActionPrototypeId,
    pub kind: ActionKind,
    name: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl ActionPrototype {
    pub fn assemble(id: ActionPrototypeId, content: ActionPrototypeContentV1) -> Self {
        Self {
            id,
            name: content.name,
            kind: content.kind,
            timestamp: content.timestamp,
        }
    }

    pub async fn new(
        ctx: &DalContext,
        name: Option<impl Into<String>>,
        kind: ActionKind,
        schema_variant_id: SchemaVariantId,
        func_id: FuncId,
    ) -> ActionPrototypeResult<Self> {
        let timestamp = Timestamp::now();

        let content = ActionPrototypeContentV1 {
            kind,
            timestamp,
            name: name.map(Into::into),
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ActionPrototypeContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ActionPrototype(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot.add_node(node_weight.to_owned()).await?;
        workspace_snapshot
            .add_edge(
                schema_variant_id,
                EdgeWeight::new(change_set, EdgeWeightKind::ActionPrototype(kind))?,
                id,
            )
            .await?;
        workspace_snapshot
            .add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                func_id,
            )
            .await?;

        Ok(ActionPrototype::assemble(id.into(), content))
    }

    pub async fn for_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ActionPrototypeResult<Vec<Self>> {
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
            content_hashes.push(weight.content_hash());
            node_weights.push(weight);
        }

        let content_map: HashMap<ContentHash, ActionPrototypeContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
            .await?;

        let mut prototypes = Vec::with_capacity(node_weights.len());
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let ActionPrototypeContent::V1(inner) = content;

                    prototypes.push(Self::assemble(node_weight.id().into(), inner.clone()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }
        Ok(prototypes)
    }

    pub async fn get_by_id(ctx: &DalContext, id: ActionPrototypeId) -> ActionPrototypeResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let ulid: ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: ActionPrototypeContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let ActionPrototypeContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    pub async fn func_id(&self, ctx: &DalContext) -> ActionPrototypeResult<FuncId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(self.id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let id = node_weight.id();
            if NodeWeightDiscriminants::Func == node_weight.into() {
                return Ok(id.into());
            }
        }

        Err(ActionPrototypeError::MissingFunction(self.id))
    }

    pub async fn run(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ActionPrototypeResult<Option<ActionRunResult>> {
        let component = Component::get_by_id(ctx, component_id).await?;
        let component_view = component.materialized_view(ctx).await?;
        let before = before_funcs_for_component(ctx, &component_id).await?;

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

        Ok(match return_value.value() {
            Some(value) => {
                let mut run_result: ActionRunResult = serde_json::from_value(value.clone())?;
                run_result.logs = logs.iter().map(|l| l.message.clone()).collect();

                let component = if component.to_delete() && run_result.payload.is_none() {
                    component.set_to_delete(ctx, false).await?
                } else {
                    component
                };

                if component.resource(ctx).await? != run_result {
                    component.set_resource(ctx, run_result.clone()).await?;
                    WsEvent::resource_refreshed(ctx, component.id())
                        .await?
                        .publish_on_commit(ctx)
                        .await?;
                }

                Some(run_result)
            }
            None => None,
        })
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
