//! This module contains [`SchemaVariant`](SchemaVariant), which is the "class" of a [`Component`](crate::Component).

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use chrono::Utc;
use petgraph::{Direction, Incoming, Outgoing};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::ParseError;

pub use json::SchemaVariantJson;
pub use json::SchemaVariantMetadataJson;
pub use metadata_view::SchemaVariantMetadataView;
use si_events::{ulid::Ulid, ContentHash};
use si_frontend_types as frontend_types;
use si_layer_cache::LayerDbError;
use si_pkg::SpecError;
use telemetry::prelude::*;
pub use value_from::ValueFrom;

use crate::action::prototype::{ActionKind, ActionPrototype};
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::value::{AttributeValueError, ValueIsFor};
use crate::change_set::ChangeSetError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::{FuncError, FuncKind};
use crate::layer_db_types::{
    ContentTypeError, InputSocketContent, OutputSocketContent, SchemaVariantContent,
    SchemaVariantContentV2,
};
use crate::module::Module;
use crate::prop::{PropError, PropPath};
use crate::schema::variant::root_prop::RootProp;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::NodeIndex;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError, PropNodeWeight};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    implement_add_edge_to, pk,
    schema::variant::leaves::{LeafInput, LeafInputLocation, LeafKind},
    ActionPrototypeId, AttributePrototype, AttributePrototypeId, ChangeSetId, ComponentId,
    ComponentType, DalContext, Func, FuncId, HelperError, InputSocket, OutputSocket,
    OutputSocketId, Prop, PropId, PropKind, Schema, SchemaError, SchemaId, Timestamp,
    TransactionsError, WsEvent, WsEventResult, WsPayload,
};
use crate::{AttributeValue, Component, ComponentError, FuncBackendResponseType, InputSocketId};

use self::root_prop::RootPropChild;

pub mod authoring;
mod json;
pub mod leaves;
mod metadata_view;
pub mod root_prop;
mod value_from;

// FIXME(nick,theo): colors should be required for all schema variants.
// There should be no default in the backend as there should always be a color.
pub const DEFAULT_SCHEMA_VARIANT_COLOR: &str = "#00b0bc";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("action prototype error: {0}")]
    ActionPrototype(String),
    #[error("asset func not found for schema variant: {0}")]
    AssetFuncNotFound(SchemaVariantId),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute argument prototype error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype not found for input socket id: {0}")]
    AttributePrototypeNotFoundForInputSocket(InputSocketId),
    #[error("attribute prototype not found for output socket id: {0}")]
    AttributePrototypeNotFoundForOutputSocket(OutputSocketId),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("content error: {0}")]
    ContentType(#[from] ContentTypeError),
    #[error("default schema variant not found for schema: {0}")]
    DefaultSchemaVariantNotFound(SchemaId),
    #[error("default variant not found: {0}")]
    DefaultVariantNotFound(String),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("{0} exists, but is not a schema variant id")]
    IdForWrongType(Ulid),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("Func {0} of response type {1} cannot set leaf {2:?}")]
    LeafFunctionMismatch(FuncId, FuncBackendResponseType, LeafKind),
    #[error("func {0} not a JsAttribute func, required for leaf functions")]
    LeafFunctionMustBeJsAttribute(FuncId),
    #[error("Leaf map prop not found for item prop {0}")]
    LeafMapPropNotFound(PropId),
    #[error("schema variant missing asset func id; schema_variant_id={0}")]
    MissingAssetFuncId(SchemaVariantId),
    #[error("module error: {0}")]
    Module(String),
    #[error("more than one schema found for schema variant: {0}")]
    MoreThanOneSchemaFound(SchemaVariantId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("schema variant not found: {0}")]
    NotFound(SchemaVariantId),
    #[error("schema variant not found for input socket: {0}")]
    NotFoundForInputSocket(InputSocketId),
    #[error("schema variant not found for output socket: {0}")]
    NotFoundForOutputSocket(OutputSocketId),
    #[error("schema variant not found for prop: {0}")]
    NotFoundForProp(PropId),
    #[error("schema variant not found for root prop: {0}")]
    NotFoundForRootProp(PropId),
    #[error("schema spec has no variants")]
    NoVariants,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("found prop id {0} that is not a prop")]
    PropIdNotAProp(PropId),
    #[error("cannot find prop at path {1} for SchemaVariant {0}")]
    PropNotFoundAtPath(SchemaVariantId, String),
    #[error("schema variant {0} has no root node")]
    RootNodeMissing(SchemaVariantId),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found for schema variant: {0}")]
    SchemaNotFound(SchemaVariantId),
    #[error("schema variant locked: {0}")]
    SchemaVariantLocked(SchemaVariantId),
    #[error("secret defining schema variant ({0}) has no output sockets and needs one for the secret corresponding to its secret definition")]
    SecretDefiningSchemaVariantMissingOutputSocket(SchemaVariantId),
    #[error("found too many output sockets ({0:?}) for secret defining schema variant ({1})")]
    SecretDefiningSchemaVariantTooManyOutputSockets(Vec<OutputSocketId>, SchemaVariantId),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("spec error: {0}")]
    Spec(#[from] SpecError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("url parse error: {0}")]
    Url(#[from] ParseError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

pk!(SchemaVariantId);

impl From<si_events::SchemaVariantId> for SchemaVariantId {
    fn from(value: si_events::SchemaVariantId) -> Self {
        Self(value.into_raw_id())
    }
}

impl From<SchemaVariantId> for si_events::SchemaVariantId {
    fn from(value: SchemaVariantId) -> Self {
        Self::from_raw_id(value.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pub id: SchemaVariantId,
    #[serde(flatten)]
    timestamp: Timestamp,
    ui_hidden: bool,
    version: String,
    display_name: String,
    category: String,
    color: String,
    component_type: ComponentType,
    link: Option<String>,
    description: Option<String>,
    asset_func_id: Option<FuncId>,
    finalized_once: bool,
    is_builtin: bool,
    is_locked: bool,
}

impl SchemaVariant {
    pub async fn into_frontend_type(
        self,
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<frontend_types::SchemaVariant> {
        // NOTE(fnichol): We're going to start asserting that there *is* an asset func id as all
        // schema variants must be created via a func. Since the graph representation currently has
        // this as optional we make this assertion here and error if not present.
        let asset_func_id = self
            .asset_func_id
            .ok_or(SchemaVariantError::MissingAssetFuncId(self.id()))?
            .into();

        let (output_sockets, input_sockets) = Self::list_all_sockets(ctx, self.id()).await?;
        let func_ids: Vec<_> = Self::all_func_ids(ctx, self.id())
            .await?
            .into_iter()
            .map(|func_id| func_id.into())
            .collect();

        let schema = Schema::get_by_id(ctx, schema_id).await?;

        let is_default = schema.get_default_schema_variant_id(ctx).await? == Some(self.id());
        let props = Self::all_props(ctx, self.id()).await?;
        let mut front_end_props = Vec::with_capacity(props.len());
        for prop in props {
            let new_prop = prop.into_frontend_type(ctx).await?;
            front_end_props.push(new_prop);
        }

        let can_contribute = Self::can_be_contributed_by_id(ctx, self.id).await?;

        Ok(frontend_types::SchemaVariant {
            schema_id: schema_id.into(),
            schema_name: schema.name().to_owned(),
            schema_variant_id: self.id.into(),
            version: self.version,
            display_name: self.display_name,
            category: self.category,
            description: self.description,
            link: self.link,
            color: self.color,
            asset_func_id,
            func_ids,
            component_type: self.component_type.into(),
            input_sockets: input_sockets
                .into_iter()
                .map(|socket| socket.into())
                .collect(),
            output_sockets: output_sockets
                .into_iter()
                .map(|socket| socket.into())
                .collect(),
            is_locked: self.is_locked,
            timestamp: self.timestamp.into(),
            props: front_end_props,
            can_create_new_components: is_default || !self.is_locked,
            can_contribute,
        })
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantClonedPayload {
    schema_variant_id: SchemaVariantId,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantUpdatedPayload {
    old_schema_variant_id: SchemaVariantId,
    new_schema_variant_id: SchemaVariantId,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantSavedPayload {
    schema_variant_id: SchemaVariantId,
    change_set_id: ChangeSetId,
    schema_id: SchemaId,
    name: String,
    category: String,
    color: String,
    component_type: ComponentType,
    link: Option<String>,
    description: Option<String>,
    display_name: String,
}
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDeletedPayload {
    schema_variant_id: SchemaVariantId,
    schema_id: SchemaId,
    change_set_id: ChangeSetId,
}

impl WsEvent {
    pub async fn schema_variant_created(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant: SchemaVariant,
    ) -> WsEventResult<Self> {
        let payload = schema_variant.into_frontend_type(ctx, schema_id).await?;
        WsEvent::new(ctx, WsPayload::SchemaVariantCreated(payload)).await
    }

    pub async fn schema_variant_updated(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariant,
    ) -> WsEventResult<Self> {
        let payload = schema_variant_id.into_frontend_type(ctx, schema_id).await?;
        WsEvent::new(ctx, WsPayload::SchemaVariantUpdated(payload)).await
    }
    pub async fn schema_variant_deleted(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SchemaVariantDeleted(SchemaVariantDeletedPayload {
                schema_variant_id,
                schema_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn schema_variant_saved(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        // TODO this should be version now
        name: String,
        category: String,
        color: String,
        component_type: ComponentType,
        link: Option<String>,
        description: Option<String>,
        display_name: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SchemaVariantSaved(SchemaVariantSavedPayload {
                schema_variant_id,
                schema_id,
                name,
                category,
                color,
                component_type,
                link,
                description,
                display_name,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn schema_variant_cloned(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SchemaVariantCloned(SchemaVariantClonedPayload {
                schema_variant_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn schema_variant_update_finished(
        ctx: &DalContext,
        old_schema_variant_id: SchemaVariantId,
        new_schema_variant_id: SchemaVariantId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SchemaVariantUpdateFinished(SchemaVariantUpdatedPayload {
                old_schema_variant_id,
                new_schema_variant_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }
}

impl From<SchemaVariant> for SchemaVariantContent {
    fn from(value: SchemaVariant) -> Self {
        Self::V2(SchemaVariantContentV2 {
            timestamp: value.timestamp(),
            ui_hidden: value.ui_hidden(),
            version: value.version().to_string(),
            display_name: value.display_name,
            category: value.category,
            color: value.color,
            component_type: value.component_type,
            link: value.link,
            description: value.description,
            asset_func_id: value.asset_func_id,
            finalized_once: value.finalized_once,
            is_builtin: value.is_builtin,
            is_locked: value.is_locked,
        })
    }
}

impl SchemaVariant {
    pub async fn assemble(
        ctx: &DalContext,
        id: SchemaVariantId,
        content: SchemaVariantContent,
    ) -> SchemaVariantResult<Self> {
        let inner = content.extract(ctx, id).await?;

        Ok(Self {
            id,
            timestamp: inner.timestamp,
            version: inner.version,
            display_name: inner.display_name,
            category: inner.category,
            color: inner.color,
            component_type: inner.component_type,
            link: inner.link,
            description: inner.description,
            asset_func_id: inner.asset_func_id,
            ui_hidden: inner.ui_hidden,
            finalized_once: inner.finalized_once,
            is_builtin: inner.is_builtin,
            is_locked: inner.is_locked,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        schema_id: SchemaId,
        version: impl Into<String>,
        display_name: impl Into<String>,
        category: impl Into<String>,
        color: impl Into<String>,
        component_type: impl Into<ComponentType>,
        link: impl Into<Option<String>>,
        description: impl Into<Option<String>>,
        asset_func_id: Option<FuncId>,
        is_builtin: bool,
    ) -> SchemaVariantResult<(Self, RootProp)> {
        debug!(%schema_id, "creating schema variant and root prop tree");
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let content = SchemaVariantContentV2 {
            timestamp: Timestamp::now(),
            version: version.into(),
            link: link.into(),
            ui_hidden: false,
            finalized_once: false,
            category: category.into(),
            color: color.into(),
            display_name: display_name.into(),
            component_type: component_type.into(),
            description: description.into(),
            asset_func_id,
            is_builtin,
            is_locked: false,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(SchemaVariantContent::V2(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;
        let node_weight = NodeWeight::new_content(
            ctx.vector_clock_id()?,
            id,
            lineage_id,
            ContentAddress::SchemaVariant(hash),
        )?;
        workspace_snapshot.add_node(node_weight).await?;

        // Schema --Use--> SchemaVariant (this)
        Schema::add_edge_to_variant(ctx, schema_id, id.into(), EdgeWeightKind::new_use()).await?;

        let schema_variant_id: SchemaVariantId = id.into();
        let root_prop = RootProp::new(ctx, schema_variant_id).await?;
        let _func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;

        let schema_variant =
            Self::assemble(ctx, id.into(), SchemaVariantContent::V2(content)).await?;
        Ok((schema_variant, root_prop))
    }

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> SchemaVariantResult<Self>
    where
        L: FnOnce(&mut Self) -> SchemaVariantResult<()>,
    {
        let mut schema_variant = self;

        let before = SchemaVariantContent::from(schema_variant.clone());
        lambda(&mut schema_variant)?;
        let updated = SchemaVariantContent::from(schema_variant.clone());

        if updated != before {
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(updated.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;

            ctx.workspace_snapshot()?
                .update_content(ctx.vector_clock_id()?, schema_variant.id.into(), hash)
                .await?;
        }

        Ok(schema_variant)
    }

    pub async fn cleanup_unlocked_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        let variant = Self::get_by_id_or_error(ctx, schema_variant_id).await?;

        if variant.is_locked() {
            return Err(SchemaVariantError::SchemaVariantLocked(schema_variant_id));
        }

        let schema = variant.schema(ctx).await?;

        // Firstly we want to delete the asset func
        let asset_func = variant.get_asset_func(ctx).await?;
        Func::delete_by_id(ctx, asset_func.id).await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(default_schema_variant_id) = schema.get_default_schema_variant_id(ctx).await? {
            if variant.id == default_schema_variant_id {
                workspace_snapshot
                    .remove_node_by_id(ctx.vector_clock_id()?, schema.id())
                    .await?;
            }
        }

        // now we want to delete the schema variant
        workspace_snapshot
            .remove_node_by_id(ctx.vector_clock_id()?, variant.id)
            .await?;

        Ok(())
    }

    pub async fn lock(self, ctx: &DalContext) -> SchemaVariantResult<SchemaVariant> {
        self.modify(ctx, |sv| {
            sv.version = Self::generate_version_string();
            sv.is_locked = true;
            Ok(())
        })
        .await
    }

    /// Returns all [`PropIds`](Prop) for a given [`SchemaVariantId`](SchemaVariant).
    pub async fn all_prop_ids(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<HashSet<PropId>> {
        let mut prop_ids = HashSet::new();

        let root_prop_id = Self::get_root_prop_id(ctx, schema_variant_id).await?;
        let mut work_queue = VecDeque::from([root_prop_id]);

        let workspace_snapshot = ctx.workspace_snapshot()?;

        while let Some(prop_id) = work_queue.pop_front() {
            let node_weight = workspace_snapshot.get_node_weight_by_id(prop_id).await?;

            // Find and load any child props.
            match node_weight {
                NodeWeight::Prop(_) => {
                    if let Some(ordering_node_idx) = workspace_snapshot
                        .outgoing_targets_for_edge_weight_kind(
                            prop_id,
                            EdgeWeightKindDiscriminants::Ordering,
                        )
                        .await?
                        .first()
                    {
                        let ordering_node_weight = workspace_snapshot
                            .get_node_weight(*ordering_node_idx)
                            .await?
                            .get_ordering_node_weight()?;

                        for &id in ordering_node_weight.order() {
                            work_queue.push_back(id.into());
                        }
                    }
                }
                _ => return Err(SchemaVariantError::PropIdNotAProp(prop_id)),
            }

            // Once processed, push onto the list that will be returned.
            prop_ids.insert(prop_id);
        }

        Ok(prop_ids)
    }

    /// Returns all [`Props`](Prop) for a given [`SchemaVariantId`](SchemaVariant).
    pub async fn all_props(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<Prop>> {
        let all_prop_ids = Self::all_prop_ids(ctx, schema_variant_id).await?;
        let all_props = Prop::list_content(ctx, all_prop_ids.into_iter().collect()).await?;
        Ok(all_props)
    }

    pub async fn get_by_id_or_error(
        ctx: &DalContext,
        id: SchemaVariantId,
    ) -> SchemaVariantResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_index = workspace_snapshot
            .get_node_index_by_id(id)
            .await
            .map_err(|err| {
                if err.is_node_with_id_not_found() {
                    SchemaVariantError::NotFound(id)
                } else {
                    err.into()
                }
            })?;

        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;

        let NodeWeight::Content(content) = &node_weight else {
            return Err(SchemaVariantError::IdForWrongType(id.into()));
        };

        let ContentAddress::SchemaVariant(hash) = content.content_address() else {
            return Err(SchemaVariantError::IdForWrongType(id.into()));
        };

        let content: SchemaVariantContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        Self::assemble(ctx, id, content).await
    }

    /// Lists all [`Components`](Component) that are using the provided [`SchemaVariantId`](SchemaVariant).
    pub async fn list_component_ids(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<ComponentId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let incoming_nodes_indices = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut component_ids: Vec<ComponentId> = Vec::new();
        for incoming_node_idx in incoming_nodes_indices {
            if let NodeWeight::Component(component_node_weight) = workspace_snapshot
                .get_node_weight(incoming_node_idx)
                .await?
            {
                component_ids.push(component_node_weight.id().into());
            }
        }
        Ok(component_ids)
    }

    pub async fn get_authoring_func(
        &self,
        ctx: &DalContext,
    ) -> SchemaVariantResult<Option<FuncId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // There's only ever 1 outgoing edge from a schema variant
        // that edge is to a FuncId
        let asset_authoring_func_node_indicies = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(self.id, EdgeWeightKindDiscriminants::Use)
            .await?;

        for asset_authoring_func_node_index in asset_authoring_func_node_indicies {
            let node_weight = workspace_snapshot
                .get_node_weight(asset_authoring_func_node_index)
                .await?;

            let func_node_weight = node_weight.get_func_node_weight()?;
            if func_node_weight.func_kind() == FuncKind::SchemaVariantDefinition {
                return Ok(Some(func_node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub async fn find_root_child_prop_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        root_prop_child: RootPropChild,
    ) -> SchemaVariantResult<PropId> {
        Ok(
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &root_prop_child.prop_path())
                .await?,
        )
    }

    /// Lists all default [`SchemaVariants`](SchemaVariant) by ID in the workspace.
    pub async fn list_default_ids(ctx: &DalContext) -> SchemaVariantResult<Vec<SchemaVariantId>> {
        let mut schema_variant_ids = Vec::new();

        for schema_id in Schema::list_ids(ctx).await? {
            schema_variant_ids.push(Self::get_default_id_for_schema(ctx, schema_id).await?);
        }

        Ok(schema_variant_ids)
    }

    pub async fn get_unlocked_for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<Option<Self>> {
        Ok(Self::list_for_schema(ctx, schema_id)
            .await?
            .into_iter()
            .find(|v| !v.is_locked))
    }

    pub async fn is_locked_by_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<bool> {
        let schema_variant = Self::get_by_id_or_error(ctx, schema_variant_id).await?;
        Ok(schema_variant.is_locked)
    }

    pub async fn error_if_locked(ctx: &DalContext, id: SchemaVariantId) -> SchemaVariantResult<()> {
        match Self::is_locked_by_id(ctx, id).await? {
            true => Err(SchemaVariantError::SchemaVariantLocked(id)),
            false => Ok(()),
        }
    }

    pub async fn can_be_contributed_by_id(
        ctx: &DalContext,
        id: SchemaVariantId,
    ) -> SchemaVariantResult<bool> {
        Ok(Self::is_default_by_id(ctx, id).await?
            && Self::is_locked_by_id(ctx, id).await?
            && Module::find_for_member_id(ctx, id)
                .await
                .map_err(|e| SchemaVariantError::Module(e.to_string()))?
                .is_none())
    }

    pub async fn get_latest_for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<Option<Self>> {
        let mut variants = Self::list_for_schema(ctx, schema_id).await?;

        variants.sort_by_key(|v| v.version().to_string());

        Ok(variants.last().cloned())
    }

    pub async fn get_default_for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<Self> {
        let default_schema_variant_id = Schema::get_default_schema_variant_by_id(ctx, schema_id)
            .await?
            .ok_or(SchemaVariantError::DefaultSchemaVariantNotFound(schema_id))?;

        Self::get_by_id_or_error(ctx, default_schema_variant_id).await
    }

    pub async fn get_default_id_for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<SchemaVariantId> {
        let default_schema_variant_id = Schema::get_default_schema_variant_by_id(ctx, schema_id)
            .await?
            .ok_or(SchemaVariantError::DefaultSchemaVariantNotFound(schema_id))?;
        Ok(default_schema_variant_id)
    }
    pub async fn list_for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut schema_variants = vec![];
        let parent_index = workspace_snapshot.get_node_index_by_id(schema_id).await?;

        // We want to use the EdgeWeightKindDiscriminants rather than EdgeWeightKind
        // this will bring us back all use and default edges!
        let node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind_by_index(
                parent_index,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut node_weights = vec![];
        let mut content_hashes = vec![];
        for index in node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::SchemaVariant)?;
            content_hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let content_map: HashMap<ContentHash, SchemaVariantContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
            .await?;

        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    schema_variants
                        .push(Self::assemble(ctx, node_weight.id().into(), content.clone()).await?);
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(schema_variants)
    }

    pub fn id(&self) -> SchemaVariantId {
        self.id
    }

    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn color(&self) -> &str {
        &self.color
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn link(&self) -> Option<String> {
        self.link.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn component_type(&self) -> ComponentType {
        self.component_type
    }

    pub fn asset_func_id(&self) -> Option<FuncId> {
        self.asset_func_id
    }

    pub fn is_builtin(&self) -> bool {
        self.is_builtin
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub async fn is_default_by_id(
        ctx: &DalContext,
        id: SchemaVariantId,
    ) -> SchemaVariantResult<bool> {
        let schema_id = Self::schema_id_for_schema_variant_id(ctx, id).await?;

        Ok(Self::get_default_id_for_schema(ctx, schema_id).await? == id)
    }

    pub async fn is_default(&self, ctx: &DalContext) -> SchemaVariantResult<bool> {
        Self::is_default_by_id(ctx, self.id).await
    }

    pub async fn get_asset_func(&self, ctx: &DalContext) -> SchemaVariantResult<Func> {
        let asset_func_id = self
            .asset_func_id
            .ok_or(SchemaVariantError::MissingAssetFuncId(self.id))?;

        Ok(Func::get_by_id_or_error(ctx, asset_func_id).await?)
    }

    pub async fn get_root_prop_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropId> {
        let root_prop_node_weight = Self::get_root_prop_node_weight(ctx, schema_variant_id).await?;
        Ok(root_prop_node_weight.id().into())
    }

    async fn get_root_prop_node_weight(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropNodeWeight> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let edge_targets: Vec<NodeIndex> = workspace_snapshot
            .edges_directed(schema_variant_id, Direction::Outgoing)
            .await?
            .into_iter()
            .map(|(_, _, target_idx)| target_idx)
            .collect();

        for index in edge_targets {
            let node_weight = workspace_snapshot.get_node_weight(index).await?;
            // TODO(nick): ensure that only one prop can be under a schema variant.
            if let NodeWeight::Prop(inner_weight) = node_weight {
                if inner_weight.name() == "root" {
                    return Ok(inner_weight.clone());
                }
            }
        }

        Err(SchemaVariantError::RootNodeMissing(schema_variant_id))
    }

    pub async fn create_default_prototypes(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        debug!(%schema_variant_id, "creating default prototypes");
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;
        let root_prop_node_weight = Self::get_root_prop_node_weight(ctx, schema_variant_id).await?;
        let mut work_queue: VecDeque<PropNodeWeight> = VecDeque::from(vec![root_prop_node_weight]);

        while let Some(prop) = work_queue.pop_front() {
            // See an attribute prototype exists.
            let mut found_attribute_prototype_id: Option<AttributePrototypeId> = None;
            {
                let targets = workspace_snapshot
                    .outgoing_targets_for_edge_weight_kind(
                        prop.id(),
                        EdgeWeightKindDiscriminants::Prototype,
                    )
                    .await?;
                for target in targets {
                    let node_weight = workspace_snapshot.get_node_weight(target).await?;
                    if let Some(ContentAddressDiscriminants::AttributePrototype) =
                        node_weight.content_address_discriminants()
                    {
                        found_attribute_prototype_id = Some(node_weight.id().into());
                        break;
                    }
                }
            }

            // Create the attribute prototype and appropriate edges if they do not exist.
            if found_attribute_prototype_id.is_none() {
                // We did not find a prototype, so we must create one.
                let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

                // New edge Prop --Prototype--> AttributePrototype.
                Prop::add_edge_to_attribute_prototype(
                    ctx,
                    prop.id().into(),
                    attribute_prototype.id(),
                    EdgeWeightKind::Prototype(None),
                )
                .await?;
            }

            // Push all children onto the work queue.
            let targets = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(prop.id(), EdgeWeightKindDiscriminants::Use)
                .await?;
            for target in targets {
                let node_weight = workspace_snapshot.get_node_weight(target).await?;
                if let NodeWeight::Prop(child_prop) = node_weight {
                    work_queue.push_back(child_prop.to_owned())
                }
            }
        }

        Ok(())
    }

    pub async fn mark_props_as_able_to_be_used_as_prototype_args(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let root_prop_node_weight = Self::get_root_prop_node_weight(ctx, schema_variant_id).await?;
        let root_prop_idx = workspace_snapshot
            .get_node_index_by_id(root_prop_node_weight.id())
            .await?;

        let mut work_queue = VecDeque::new();
        work_queue.push_back(root_prop_idx);

        while let Some(prop_idx) = work_queue.pop_front() {
            workspace_snapshot
                .mark_prop_as_able_to_be_used_as_prototype_arg(prop_idx)
                .await?;

            let node_weight = workspace_snapshot
                .get_node_weight(prop_idx)
                .await?
                .to_owned();
            if let NodeWeight::Prop(prop) = node_weight {
                // Only descend if we are an object.
                if prop.kind() == PropKind::Object {
                    let targets = workspace_snapshot
                        .outgoing_targets_for_edge_weight_kind(
                            prop.id(),
                            EdgeWeightKindDiscriminants::Use,
                        )
                        .await?;
                    work_queue.extend(targets);
                }
            }
        }

        Ok(())
    }

    implement_add_edge_to!(
        source_id: SchemaVariantId,
        destination_id: PropId,
        add_fn: add_edge_to_prop,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: SchemaVariantResult,
    );
    implement_add_edge_to!(
        source_id: SchemaVariantId,
        destination_id: FuncId,
        add_fn: add_edge_to_authentication_func,
        discriminant: EdgeWeightKindDiscriminants::AuthenticationPrototype,
        result: SchemaVariantResult,
    );
    implement_add_edge_to!(
        source_id: SchemaVariantId,
        destination_id: InputSocketId,
        add_fn: add_edge_to_input_socket,
        discriminant: EdgeWeightKindDiscriminants::Socket,
        result: SchemaVariantResult,
    );
    implement_add_edge_to!(
        source_id: SchemaVariantId,
        destination_id: OutputSocketId,
        add_fn: add_edge_to_output_socket,
        discriminant: EdgeWeightKindDiscriminants::Socket,
        result: SchemaVariantResult,
    );
    implement_add_edge_to!(
        source_id: SchemaVariantId,
        destination_id: ActionPrototypeId,
        add_fn: add_edge_to_action_prototype,
        discriminant: EdgeWeightKindDiscriminants::ActionPrototype,
        result: SchemaVariantResult,
    );

    pub async fn find_action_prototypes_by_kind(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        kind: ActionKind,
    ) -> SchemaVariantResult<Vec<ActionPrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id: Ulid = schema_variant_id.into();

        let action_prototype_node_idxs = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::ActionPrototype)
            .await?;

        let mut prototype_ids = vec![];

        for prototype_idx in action_prototype_node_idxs {
            let NodeWeight::ActionPrototype(weight) =
                workspace_snapshot.get_node_weight(prototype_idx).await?
            else {
                continue;
            };

            if weight.kind() == kind {
                prototype_ids.push(weight.id().into());
            }
        }

        Ok(prototype_ids)
    }

    pub async fn find_leaf_item_functions(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
    ) -> SchemaVariantResult<Vec<FuncId>> {
        let mut func_ids = vec![];
        let leaf_item_prop_id =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

        for (maybe_key, proto) in Prop::prototypes_by_key(ctx, leaf_item_prop_id).await? {
            if maybe_key.is_some() {
                let func_id = AttributePrototype::func_id(ctx, proto).await?;
                func_ids.push(func_id);
            }
        }

        Ok(func_ids)
    }
    pub async fn new_authentication_prototype(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        Self::add_edge_to_authentication_func(
            ctx,
            schema_variant_id,
            func_id,
            EdgeWeightKind::AuthenticationPrototype,
        )
        .await?;
        Ok(())
    }

    pub async fn remove_authentication_prototype(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        ctx.workspace_snapshot()?
            .remove_edge_for_ulids(
                ctx.vector_clock_id()?,
                schema_variant_id,
                func_id,
                EdgeWeightKindDiscriminants::AuthenticationPrototype,
            )
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn get_content(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<(ContentHash, SchemaVariantContentV2)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id: Ulid = schema_variant_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: SchemaVariantContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        Ok((hash, content.extract(ctx, schema_variant_id).await?))
    }

    /// This _idempotent_ function "finalizes" a [`SchemaVariant`].
    ///
    /// This method **MUST** be called once all the [`Props`](Prop) have been created for the
    /// [`SchemaVariant`]. It can be called multiple times while [`Props`](Prop) are being created,
    /// but it must be called once after all [`Props`](Prop) have been created.
    pub async fn finalize(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        Self::create_default_prototypes(ctx, schema_variant_id).await?;
        Self::mark_props_as_able_to_be_used_as_prototype_args(ctx, schema_variant_id).await?;

        // TODO(nick,jacob,zack): if we are going to copy the existing system (which we likely will), we need to
        // set "/root/si/type" and "/root/si/protected".

        Ok(())
    }

    pub async fn get_color(&self, ctx: &DalContext) -> SchemaVariantResult<String> {
        let color_prop_id =
            Prop::find_prop_id_by_path(ctx, self.id, &PropPath::new(["root", "si", "color"]))
                .await?;

        let prototype_id = Prop::prototype_id(ctx, color_prop_id).await?;

        match AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id)
            .await?
            .first()
        {
            None => Ok(DEFAULT_SCHEMA_VARIANT_COLOR.to_string()),
            Some(apa_id) => {
                match AttributePrototypeArgument::static_value_by_id(ctx, *apa_id).await? {
                    Some(static_value) => {
                        let color: String = serde_json::from_value(static_value.value)?;
                        Ok(color)
                    }
                    None => Ok(DEFAULT_SCHEMA_VARIANT_COLOR.to_string()),
                }
            }
        }
    }

    /// Configures the "default" value for the
    /// [`AttributePrototypeArgument`](crate::attribute::prototype::argument::AttributePrototypeArgument)
    /// for the /root/si/color [`Prop`](crate::Prop). If a prototype already
    /// exists pointing to a function other than
    /// [`IntrinsicFunc::SetString`](crate::func::intrinsics::IntrinsicFunc::SetString)
    /// we will remove that edge and replace it with one pointing to
    /// `SetString`.
    pub async fn set_color(
        &self,
        ctx: &DalContext,
        color: impl AsRef<str>,
    ) -> SchemaVariantResult<()> {
        let color_prop_id =
            Prop::find_prop_id_by_path(ctx, self.id, &PropPath::new(["root", "si", "color"]))
                .await?;

        Prop::set_default_value(ctx, color_prop_id, color.as_ref()).await?;

        Ok(())
    }

    /// Configures the "default" value for the
    /// [`AttributePrototypeArgument`](crate::attribute::prototype::argument::AttributePrototypeArgument)
    /// for the /root/si/type [`Prop`](crate::Prop). If a prototype already
    /// exists pointing to a function other than
    /// [`IntrinsicFunc::SetString`](crate::func::intrinsics::IntrinsicFunc::SetString)
    /// we will remove that edge and replace it with one pointing to
    /// `SetString`.
    pub async fn set_type(
        &self,
        ctx: &DalContext,
        component_type: impl AsRef<str>,
    ) -> SchemaVariantResult<()> {
        let type_prop_id =
            Prop::find_prop_id_by_path(ctx, self.id, &PropPath::new(["root", "si", "type"]))
                .await?;

        Prop::set_default_value(ctx, type_prop_id, component_type.as_ref()).await?;

        Ok(())
    }

    /// Configures the "default" value for the
    /// [`AttributePrototypeArgument`](crate::attribute::prototype::argument::AttributePrototypeArgument)
    /// for the /root/si/type [`Prop`](crate::Prop). If a prototype already
    /// exists pointing to a function other than
    /// [`IntrinsicFunc::SetString`](crate::func::intrinsics::IntrinsicFunc::SetString)
    /// we will remove that edge and replace it with one pointing to
    /// `SetString`.
    pub async fn get_type(&self, ctx: &DalContext) -> SchemaVariantResult<Option<ComponentType>> {
        let type_prop_id =
            Prop::find_prop_id_by_path(ctx, self.id, &PropPath::new(["root", "si", "type"]))
                .await?;

        let prototype_id = Prop::prototype_id(ctx, type_prop_id).await?;

        match AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id)
            .await?
            .first()
        {
            None => Ok(None),
            Some(apa_id) => {
                match AttributePrototypeArgument::static_value_by_id(ctx, *apa_id).await? {
                    Some(static_value) => {
                        let comp_type: ComponentType = serde_json::from_value(static_value.value)?;
                        Ok(Some(comp_type))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    /// This method finds a [`leaf`](crate::schema::variant::leaves)'s entry
    /// [`Prop`](crate::Prop) given a [`LeafKind`](crate::schema::variant::leaves::LeafKind).
    pub async fn find_leaf_item_prop(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
    ) -> SchemaVariantResult<PropId> {
        let (leaf_map_prop_name, leaf_item_prop_name) = leaf_kind.prop_names();

        Ok(Prop::find_prop_id_by_path(
            ctx,
            schema_variant_id,
            &PropPath::new(["root", leaf_map_prop_name, leaf_item_prop_name]),
        )
        .await?)
    }

    pub async fn upsert_leaf_function(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
        input_locations: &[LeafInputLocation],
        func: &Func,
    ) -> SchemaVariantResult<AttributePrototypeId> {
        let leaf_item_prop_id =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

        let key = Some(func.name.to_owned());

        let mut existing_args = FuncArgument::list_for_func(ctx, func.id).await?;
        let mut inputs = vec![];
        for location in input_locations {
            let arg_name = location.arg_name();
            let arg = match existing_args
                .iter()
                .find(|arg| arg.name.as_str() == arg_name)
            {
                Some(existing_arg) => existing_arg.clone(),
                None => {
                    FuncArgument::new(ctx, arg_name, location.arg_kind(), None, func.id).await?
                }
            };

            inputs.push(LeafInput {
                location: *location,
                func_argument_id: arg.id,
            });
        }

        for existing_arg in existing_args.drain(..) {
            if !inputs.iter().any(
                |&LeafInput {
                     func_argument_id, ..
                 }| func_argument_id == existing_arg.id,
            ) {
                FuncArgument::remove(ctx, existing_arg.id).await?;
            }
        }

        let attribute_prototype_id =
            match AttributePrototype::find_for_prop(ctx, leaf_item_prop_id, &key).await? {
                Some(existing_proto_id) => {
                    Self::upsert_leaf_function_inputs(
                        ctx,
                        inputs.as_slice(),
                        existing_proto_id,
                        schema_variant_id,
                    )
                    .await?;
                    existing_proto_id
                }
                None => {
                    let (_, new_proto) =
                        SchemaVariant::add_leaf(ctx, func.id, schema_variant_id, leaf_kind, inputs)
                            .await?;
                    // Before returning the new prototype, we need to ensure all existing components use the new
                    // leaf prop (either qualification or code gen).
                    let mut new_attribute_value_ids = Vec::new();
                    for component_id in Self::list_component_ids(ctx, schema_variant_id).await? {
                        let parent_attribute_value_id =
                            Component::find_map_attribute_value_for_leaf_kind(
                                ctx,
                                component_id,
                                leaf_kind,
                            )
                            .await
                            .map_err(Box::new)?;

                        let new_attribute_value = AttributeValue::new(
                            ctx,
                            ValueIsFor::Prop(leaf_item_prop_id),
                            Some(component_id),
                            Some(parent_attribute_value_id),
                            key.clone(),
                        )
                        .await
                        .map_err(Box::new)?;
                        new_attribute_value_ids.push(new_attribute_value.id);
                    }
                    if !new_attribute_value_ids.is_empty() {
                        ctx.add_dependent_values_and_enqueue(new_attribute_value_ids)
                            .await?;
                    }

                    new_proto
                }
            };

        Ok(attribute_prototype_id)
    }

    /// This method upserts [inputs](LeafInput) to an _existing_ leaf function.
    pub async fn upsert_leaf_function_inputs(
        ctx: &DalContext,
        inputs: &[LeafInput],
        attribute_prototype_id: AttributePrototypeId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        let apas =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;

        let mut apa_func_arg_ids = HashMap::new();
        for input in inputs {
            let mut exisiting_func_arg = None;
            for apa_id in &apas {
                let func_arg_id =
                    AttributePrototypeArgument::func_argument_id_by_id(ctx, *apa_id).await?;
                apa_func_arg_ids.insert(apa_id, func_arg_id);

                if func_arg_id == input.func_argument_id {
                    exisiting_func_arg = Some(func_arg_id);
                }
            }

            if exisiting_func_arg.is_none() {
                let input_prop_id =
                    Self::find_root_child_prop_id(ctx, schema_variant_id, input.location.into())
                        .await?;

                debug!(
                    %input_prop_id, ?input.location, "adding root child attribute prototype argument",
                );

                let new_apa = AttributePrototypeArgument::new(
                    ctx,
                    attribute_prototype_id,
                    input.func_argument_id,
                )
                .await?;
                new_apa.set_value_from_prop_id(ctx, input_prop_id).await?;
            }
        }

        for (apa_id, func_arg_id) in apa_func_arg_ids {
            if !inputs.iter().any(
                |&LeafInput {
                     func_argument_id, ..
                 }| { func_argument_id == func_arg_id },
            ) {
                AttributePrototypeArgument::remove(ctx, *apa_id).await?;
            }
        }

        Ok(())
    }

    pub async fn list_all_sockets(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<(Vec<OutputSocket>, Vec<InputSocket>)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // Look for all output and input sockets that the schema variant uses.
        let maybe_socket_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;

        // Collect the output and the input sockets separately.
        let mut output_socket_hashes: Vec<(OutputSocketId, ContentHash)> = Vec::new();
        let mut input_socket_hashes: Vec<(InputSocketId, ContentHash)> = Vec::new();

        for maybe_socket_node_index in maybe_socket_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(maybe_socket_node_index)
                .await?;
            if let NodeWeight::Content(content_node_weight) = node_weight {
                match content_node_weight.content_address() {
                    ContentAddress::OutputSocket(output_socket_content_hash) => {
                        output_socket_hashes
                            .push((content_node_weight.id().into(), output_socket_content_hash));
                    }
                    ContentAddress::InputSocket(input_socket_content_hash) => {
                        input_socket_hashes
                            .push((content_node_weight.id().into(), input_socket_content_hash));
                    }
                    _ => {}
                }
            }
        }

        // Grab all the contents in bulk from the content store.
        let output_socket_hashes_only: Vec<ContentHash> =
            output_socket_hashes.iter().map(|(_, h)| *h).collect();
        let output_socket_content_map: HashMap<ContentHash, OutputSocketContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(&output_socket_hashes_only)
            .await?;

        let input_socket_hashes_only: Vec<ContentHash> =
            input_socket_hashes.iter().map(|(_, h)| *h).collect();
        let input_socket_content_map: HashMap<ContentHash, InputSocketContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(&input_socket_hashes_only)
            .await?;

        // Assemble all output sockets.
        let mut output_sockets = Vec::with_capacity(output_socket_hashes.len());
        for (output_socket_id, output_socket_hash) in output_socket_hashes {
            let output_socket_content = output_socket_content_map.get(&output_socket_hash).ok_or(
                WorkspaceSnapshotError::MissingContentFromStore(output_socket_id.into()),
            )?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let OutputSocketContent::V1(output_socket_content_inner) = output_socket_content;

            output_sockets.push(OutputSocket::assemble(
                output_socket_id,
                output_socket_content_inner.to_owned(),
            ));
        }

        // Assemble all input sockets.
        let mut input_sockets = Vec::with_capacity(input_socket_hashes.len());
        for (input_socket_id, input_socket_hash) in input_socket_hashes {
            let input_socket_content = input_socket_content_map.get(&input_socket_hash).ok_or(
                WorkspaceSnapshotError::MissingContentFromStore(input_socket_id.into()),
            )?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let InputSocketContent::V1(input_socket_content_inner) = input_socket_content;

            input_sockets.push(InputSocket::assemble(
                input_socket_id,
                input_socket_content_inner.to_owned(),
            ));
        }

        Ok((output_sockets, input_sockets))
    }

    pub async fn list_all_socket_ids(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<(Vec<OutputSocketId>, Vec<InputSocketId>)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let maybe_socket_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;

        let mut output_socket_ids: Vec<OutputSocketId> = Vec::new();
        let mut input_socket_ids: Vec<InputSocketId> = Vec::new();

        for maybe_socket_node_index in maybe_socket_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(maybe_socket_node_index)
                .await?;
            if let Some(content_address_discriminant) = node_weight.content_address_discriminants()
            {
                match content_address_discriminant {
                    ContentAddressDiscriminants::InputSocket => {
                        input_socket_ids.push(node_weight.id().into())
                    }
                    ContentAddressDiscriminants::OutputSocket => {
                        output_socket_ids.push(node_weight.id().into())
                    }
                    _ => {}
                }
            }
        }

        Ok((output_socket_ids, input_socket_ids))
    }

    pub async fn schema(&self, ctx: &DalContext) -> SchemaVariantResult<Schema> {
        Self::schema_for_schema_variant_id(ctx, self.id).await
    }

    pub async fn schema_for_schema_variant_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Schema> {
        let schema_id = Self::schema_id_for_schema_variant_id(ctx, schema_variant_id).await?;

        Ok(Schema::get_by_id(ctx, schema_id).await?)
    }

    pub async fn schema_id_for_schema_variant_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<SchemaId> {
        let schema_id = {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            let maybe_schema_indices = workspace_snapshot
                .edges_directed(schema_variant_id, Incoming)
                .await?;

            let mut schema_id: Option<SchemaId> = None;
            for (edge_weight, source_index, _target_index) in maybe_schema_indices {
                if *edge_weight.kind() == EdgeWeightKind::new_use()
                    || *edge_weight.kind() == EdgeWeightKind::new_use_default()
                {
                    if let NodeWeight::Content(content) =
                        workspace_snapshot.get_node_weight(source_index).await?
                    {
                        let content_hash_discriminants: ContentAddressDiscriminants =
                            content.content_address().into();
                        if let ContentAddressDiscriminants::Schema = content_hash_discriminants {
                            schema_id = match schema_id {
                                None => Some(content.id().into()),
                                Some(_already_found_schema_id) => {
                                    return Err(SchemaVariantError::MoreThanOneSchemaFound(
                                        schema_variant_id,
                                    ));
                                }
                            };
                        }
                    }
                }
            }
            schema_id.ok_or(SchemaVariantError::SchemaNotFound(schema_variant_id))?
        };

        Ok(schema_id)
    }

    /// Returns all [`Funcs`](Func) for a given [`SchemaVariantId`](SchemaVariant) barring
    /// [intrinsics](IntrinsicFunc).
    pub async fn all_funcs(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<Func>> {
        let func_id_set = Self::all_func_ids(ctx, schema_variant_id).await?;
        let func_ids = Vec::from_iter(func_id_set.into_iter());
        let funcs: Vec<Func> = Func::list_from_ids(ctx, func_ids.as_slice()).await?;

        // Filter out intrinsic funcs. kkep this here
        let mut filtered_funcs = Vec::new();
        for func in &funcs {
            if IntrinsicFunc::maybe_from_str(&func.name).is_none() {
                filtered_funcs.push(func.to_owned());
            }
        }

        Ok(filtered_funcs)
    }

    /// Returns all [`FuncIds`](Func) used for a given [`SchemaVariantId`](SchemaVariant).
    pub async fn all_func_ids(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<HashSet<FuncId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut all_func_ids = HashSet::new();

        // Gather all funcs for props.
        let prop_list = Self::all_prop_ids(ctx, schema_variant_id).await?;
        for prop_id in prop_list {
            let keys_and_prototypes = Prop::prototypes_by_key(ctx, prop_id).await?;
            for (_, attribute_prototype_id) in keys_and_prototypes {
                let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
                all_func_ids.insert(func_id);
            }
        }

        // Gather all funcs for sockets.
        let (output_socket_ids, input_socket_ids) =
            Self::list_all_socket_ids(ctx, schema_variant_id).await?;
        for output_socket_id in output_socket_ids {
            if let Some(attribute_prototype_id) =
                AttributePrototype::find_for_output_socket(ctx, output_socket_id).await?
            {
                let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
                all_func_ids.insert(func_id);
            }
        }
        for input_socket_id in input_socket_ids {
            if let Some(attribute_prototype_id) =
                AttributePrototype::find_for_input_socket(ctx, input_socket_id).await?
            {
                let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
                all_func_ids.insert(func_id);
            }
        }

        // Gather all auth funcs.
        let auth_func_ids = Self::list_auth_func_ids_for_id(ctx, schema_variant_id).await?;
        all_func_ids.extend(auth_func_ids);

        // Gather all action funcs.
        let action_prototype_nodes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::ActionPrototype,
            )
            .await?;
        for action_prototype_node in action_prototype_nodes {
            let node_weight = workspace_snapshot
                .get_node_weight(action_prototype_node)
                .await?;

            if let NodeWeight::ActionPrototype(action_prototype_node_weight) = node_weight {
                let func_id =
                    ActionPrototype::func_id(ctx, action_prototype_node_weight.id().into())
                        .await
                        .map_err(|err| SchemaVariantError::ActionPrototype(err.to_string()))?;

                all_func_ids.insert(func_id);
            }
        }

        Ok(all_func_ids)
    }

    pub async fn list_auth_func_ids_for_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<FuncId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut auth_func_ids = vec![];

        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::AuthenticationPrototype,
            )
            .await?
        {
            auth_func_ids.push(
                workspace_snapshot
                    .get_node_weight(node_index)
                    .await?
                    .id()
                    .into(),
            )
        }

        Ok(auth_func_ids)
    }

    pub async fn list_schema_variant_ids_using_auth_func_id(
        ctx: &DalContext,
        auth_func_id: FuncId,
    ) -> SchemaVariantResult<Vec<SchemaVariantId>> {
        let mut results = vec![];

        for schema_variant_id in Self::list_default_ids(ctx).await? {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            let targets = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    schema_variant_id,
                    EdgeWeightKindDiscriminants::AuthenticationPrototype,
                )
                .await?;

            for target in targets {
                let func_node_weight = workspace_snapshot
                    .get_node_weight(target)
                    .await?
                    .get_func_node_weight()?;
                if auth_func_id == func_node_weight.id().into() {
                    results.push(schema_variant_id);
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Find the [`SchemaVariantId`](SchemaVariant) for the given [`PropId`](Prop).
    pub async fn find_for_prop_id(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> SchemaVariantResult<SchemaVariantId> {
        let mut work_queue = VecDeque::new();
        work_queue.push_back(prop_id);

        // If the parent prop id is empty, then we know the parent is the schema variant.
        while let Some(prop_id) = work_queue.pop_front() {
            match Prop::parent_prop_id_by_id(ctx, prop_id).await? {
                Some(parent) => work_queue.push_back(parent),
                None => return Self::find_for_root_prop_id(ctx, prop_id).await,
            }
        }

        // This should be impossible to hit.
        Err(SchemaVariantError::NotFoundForProp(prop_id))
    }

    async fn find_for_root_prop_id(
        ctx: &DalContext,
        root_prop_id: PropId,
    ) -> SchemaVariantResult<SchemaVariantId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(root_prop_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        for source in sources {
            let node_weight = workspace_snapshot.get_node_weight(source).await?;
            if let Some(ContentAddressDiscriminants::SchemaVariant) =
                &node_weight.content_address_discriminants()
            {
                return Ok(node_weight.id().into());
            }
        }

        Err(SchemaVariantError::NotFoundForRootProp(root_prop_id))
    }

    /// Find the [`SchemaVariantId`](SchemaVariant) for the given [`InputSocketId`](InputSocket).
    pub async fn find_for_input_socket_id(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
    ) -> SchemaVariantResult<SchemaVariantId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                input_socket_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;

        for source in sources {
            let node_weight = workspace_snapshot.get_node_weight(source).await?;
            if let Some(ContentAddressDiscriminants::SchemaVariant) =
                &node_weight.content_address_discriminants()
            {
                return Ok(node_weight.id().into());
            }
        }

        Err(SchemaVariantError::NotFoundForInputSocket(input_socket_id))
    }

    /// Find the [`SchemaVariantId`](SchemaVariant) for the given [`OutputSocketId`](OutputSocket).
    pub async fn find_for_output_socket_id(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
    ) -> SchemaVariantResult<SchemaVariantId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                output_socket_id,
                EdgeWeightKindDiscriminants::Socket,
            )
            .await?;

        for source in sources {
            let node_weight = workspace_snapshot.get_node_weight(source).await?;
            if let Some(ContentAddressDiscriminants::SchemaVariant) =
                &node_weight.content_address_discriminants()
            {
                return Ok(node_weight.id().into());
            }
        }

        Err(SchemaVariantError::NotFoundForOutputSocket(
            output_socket_id,
        ))
    }

    /// List all [`SchemaVariantIds`](SchemaVariant) for the provided
    /// [authentication](FuncKind::Authentication) [`Func`].
    pub async fn list_for_auth_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> SchemaVariantResult<Vec<SchemaVariantId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut schema_variant_ids = vec![];

        for node_id in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                func_id,
                EdgeWeightKindDiscriminants::AuthenticationPrototype,
            )
            .await?
        {
            schema_variant_ids.push(
                workspace_snapshot
                    .get_node_weight(node_id)
                    .await?
                    .id()
                    .into(),
            )
        }

        Ok(schema_variant_ids)
    }

    /// List all [`SchemaVariantIds`](SchemaVariant) with their [`ActionPrototypes`](ActionPrototype) corresponding to
    /// the provided [action](FuncKind::Action) [`FuncId`](Func).
    ///
    /// _Note: [`SchemaVariantIds`](SchemaVariant) are not de-duplicated and can appear multiple times for different
    /// [`ActionPrototypes`](ActionPrototype).
    pub async fn list_with_action_prototypes_for_action_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> SchemaVariantResult<Vec<(SchemaVariantId, ActionPrototypeId)>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // First, collect all the action prototypes using the func.
        let mut action_prototype_raw_ids = Vec::new();
        for node_index in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            if let NodeWeight::ActionPrototype(inner) = node_weight {
                action_prototype_raw_ids.push(inner.id());
            }
        }

        // Second, collect all the schema variants alongside the action prototypes.
        let mut pairs = Vec::new();
        for action_prototype_raw_id in action_prototype_raw_ids {
            for node_index in workspace_snapshot
                .incoming_sources_for_edge_weight_kind(
                    action_prototype_raw_id,
                    EdgeWeightKindDiscriminants::ActionPrototype,
                )
                .await?
            {
                let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
                if let Some(ContentAddressDiscriminants::SchemaVariant) =
                    node_weight.content_address_discriminants()
                {
                    pairs.push((node_weight.id().into(), action_prototype_raw_id.into()));
                }
            }
        }

        Ok(pairs)
    }

    /// Remove all direct nodes connected to the schema variant, while being careful to not
    /// change anything outside the variant
    /// TODO: Implement and actual "delete schema variant function"
    pub async fn remove_external_connections(&self, ctx: &DalContext) -> SchemaVariantResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let maybe_schema_indices = workspace_snapshot.edges_directed(self.id, Outgoing).await?;

        for (edge_weight, source_index, target_index) in maybe_schema_indices {
            let kind = EdgeWeightKindDiscriminants::from(edge_weight.kind());
            match kind {
                EdgeWeightKindDiscriminants::Use
                | EdgeWeightKindDiscriminants::Socket
                | EdgeWeightKindDiscriminants::ActionPrototype => {
                    workspace_snapshot
                        .remove_all_edges(
                            ctx.vector_clock_id()?,
                            workspace_snapshot.get_node_weight(target_index).await?.id(),
                        )
                        .await?;
                }

                EdgeWeightKindDiscriminants::AuthenticationPrototype => {
                    workspace_snapshot
                        .remove_edge(ctx.vector_clock_id()?, source_index, target_index, kind)
                        .await?;
                }

                EdgeWeightKindDiscriminants::Action
                | EdgeWeightKindDiscriminants::Contain
                | EdgeWeightKindDiscriminants::FrameContains
                | EdgeWeightKindDiscriminants::Ordering
                | EdgeWeightKindDiscriminants::Ordinal
                | EdgeWeightKindDiscriminants::Prop
                | EdgeWeightKindDiscriminants::Prototype
                | EdgeWeightKindDiscriminants::PrototypeArgument
                | EdgeWeightKindDiscriminants::PrototypeArgumentValue
                | EdgeWeightKindDiscriminants::Proxy
                | EdgeWeightKindDiscriminants::Root
                | EdgeWeightKindDiscriminants::SocketValue
                | EdgeWeightKindDiscriminants::ValidationOutput => {}
            }
        }

        Ok(())
    }

    pub async fn rebuild_variant_root_prop(&self, ctx: &DalContext) -> SchemaVariantResult<()> {
        RootProp::new(ctx, self.id).await?;
        Ok(())
    }

    pub fn generate_version_string() -> String {
        format!("{}", Utc::now().format("%Y%m%d%H%M%S%f"))
    }

    /// Lists all default [`SchemaVariantIds`](SchemaVariant) that have a secret definition.
    pub async fn list_default_secret_defining_ids(
        ctx: &DalContext,
    ) -> SchemaVariantResult<Vec<SchemaVariantId>> {
        let mut ids = Vec::new();
        for maybe_secret_defining_id in Self::list_default_ids(ctx).await? {
            if Self::is_secret_defining(ctx, maybe_secret_defining_id).await? {
                ids.push(maybe_secret_defining_id);
            }
        }
        Ok(ids)
    }

    /// Finds the secret [`OutputSocket`] corresponding to the secret definition. There should only be one
    /// [`OutputSocket`] as secret defining [`SchemaVariants`](SchemaVariant) are required to have one and only one.
    ///
    /// _Warning: this function does not validate that the [`SchemaVariantId`](SchemaVariant) passed in corresponds to
    /// an actual secret defining [`SchemaVariant`]. Knowing that is the responsibility of the caller._
    pub async fn find_output_socket_for_secret_defining_id(
        ctx: &DalContext,
        secret_defining_id: SchemaVariantId,
    ) -> SchemaVariantResult<OutputSocket> {
        let (output_sockets, _) = SchemaVariant::list_all_sockets(ctx, secret_defining_id).await?;
        if output_sockets.len() > 1 {
            let output_socket_ids: Vec<OutputSocketId> =
                output_sockets.iter().map(|s| s.id()).collect();
            return Err(
                SchemaVariantError::SecretDefiningSchemaVariantTooManyOutputSockets(
                    output_socket_ids,
                    secret_defining_id,
                ),
            );
        }
        let secret_output_socket = output_sockets.first().ok_or(
            SchemaVariantError::SecretDefiningSchemaVariantMissingOutputSocket(secret_defining_id),
        )?;
        Ok(secret_output_socket.to_owned())
    }

    /// Determines if the given [`SchemaVariant`] defines a [`Secret`](crate::Secret).
    pub async fn is_secret_defining(
        ctx: &DalContext,
        id: SchemaVariantId,
    ) -> SchemaVariantResult<bool> {
        Ok(
            Prop::find_prop_id_by_path_opt(ctx, id, &PropPath::new(["root", "secret_definition"]))
                .await?
                .is_some(),
        )
    }

    /// This function lists all [`SchemaVariants`](frontend_types::SchemaVariant) for user-facing applications.
    pub async fn list_user_facing(
        ctx: &DalContext,
    ) -> SchemaVariantResult<Vec<frontend_types::SchemaVariant>> {
        let mut schema_variants = HashMap::new();

        for schema_id in Schema::list_ids(ctx).await? {
            let default_schema_variant = Self::get_default_for_schema(ctx, schema_id).await?;
            if !default_schema_variant.ui_hidden() {
                schema_variants.insert(
                    default_schema_variant.id,
                    default_schema_variant
                        .into_frontend_type(ctx, schema_id)
                        .await?,
                );
            }

            if let Some(unlocked) = Self::get_unlocked_for_schema(ctx, schema_id).await? {
                if !unlocked.ui_hidden() {
                    schema_variants.insert(
                        unlocked.id,
                        unlocked.into_frontend_type(ctx, schema_id).await?,
                    );
                }
            }

            for schema_variant in Self::list_for_schema(ctx, schema_id).await? {
                if !Self::list_component_ids(ctx, schema_variant.id())
                    .await?
                    .is_empty()
                {
                    schema_variants.insert(
                        schema_variant.id,
                        schema_variant.into_frontend_type(ctx, schema_id).await?,
                    );
                }
            }
        }

        Ok(schema_variants.into_values().collect())
    }
}
