//! This module contains [`SchemaVariant`](crate::SchemaVariant), which is t/he "class" of a
//! [`Component`](crate::Component).

use petgraph::{Direction, Incoming, Outgoing};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use si_pkg::{
    AttrFuncInputSpec, MapKeyFuncSpec, PropSpec, SchemaSpec, SchemaSpecData, SchemaVariantSpec,
    SchemaVariantSpecData, SiPropFuncSpec, SiPropFuncSpecKind, SocketSpec, SocketSpecArity,
    SocketSpecData, SocketSpecKind, SpecError,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;
use url::ParseError;

use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set::ChangeSetError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::{FuncError, FuncKind};
use crate::layer_db_types::{
    InputSocketContent, OutputSocketContent, SchemaVariantContent,
    SchemaVariantContentDiscriminants, SchemaVariantContentV1,
};
use crate::prop::{PropError, PropPath};
use crate::schema::variant::root_prop::RootProp;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::NodeIndex;

use crate::property_editor::schema::WidgetKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError, PropNodeWeight};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    implement_add_edge_to, pk,
    schema::variant::leaves::{LeafInput, LeafInputLocation, LeafKind},
    ActionPrototype, ActionPrototypeId, AttributePrototype, AttributePrototypeId, ComponentId,
    ComponentType, DalContext, Func, FuncId, HelperError, InputSocket, OutputSocket,
    OutputSocketId, Prop, PropId, PropKind, Schema, SchemaError, SchemaId, SocketArity, Timestamp,
    TransactionsError,
};
use crate::{FuncBackendResponseType, InputSocketId};

use self::root_prop::RootPropChild;

pub mod leaves;
pub mod root_prop;

// FIXME(nick,theo): colors should be required for all schema variants.
// There should be no default in the backend as there should always be a color.
pub const DEFAULT_SCHEMA_VARIANT_COLOR: &str = "#00b0bc";
pub const SCHEMA_VARIANT_VERSION: SchemaVariantContentDiscriminants =
    SchemaVariantContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("action prototype error: {0}")]
    ActionPrototype(String),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute argument prototype error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
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
    #[error("more than one schema found for schema variant: {0}")]
    MoreThanOneSchemaFound(SchemaVariantId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("schema spec has no variants")]
    NoVariants,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("found prop id {0} that is not a prop")]
    PropIdNotAProp(PropId),
    #[error("schema variant {0} has no root node")]
    RootNodeMissing(SchemaVariantId),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found for schema variant: {0}")]
    SchemaNotFound(SchemaVariantId),
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    id: SchemaVariantId,
    #[serde(flatten)]
    timestamp: Timestamp,
    ui_hidden: bool,
    name: String,
    display_name: Option<String>,
    category: String,
    color: String,
    component_type: ComponentType,
    link: Option<String>,
    description: Option<String>,
    asset_func_id: Option<FuncId>,
    finalized_once: bool,
}

impl SchemaVariant {
    pub fn assemble(id: SchemaVariantId, inner: SchemaVariantContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            display_name: inner.display_name,
            category: inner.category,
            color: inner.color,
            component_type: inner.component_type,
            link: inner.link,
            description: inner.description,
            asset_func_id: inner.asset_func_id,
            ui_hidden: inner.ui_hidden,
            finalized_once: inner.finalized_once,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        schema_id: SchemaId,
        name: impl Into<String>,
        display_name: impl Into<Option<String>>,
        category: impl Into<String>,
        color: impl Into<String>,
        component_type: impl Into<ComponentType>,
        link: impl Into<Option<String>>,
        description: impl Into<Option<String>>,
        asset_func_id: Option<FuncId>,
    ) -> SchemaVariantResult<(Self, RootProp)> {
        debug!(%schema_id, "creating schema variant and root prop tree");
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let content = SchemaVariantContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            link: link.into(),
            ui_hidden: false,
            finalized_once: false,
            category: category.into(),
            color: color.into(),
            display_name: display_name.into(),
            component_type: component_type.into(),
            description: description.into(),
            asset_func_id,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(SchemaVariantContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::SchemaVariant(hash))?;
        workspace_snapshot.add_node(node_weight).await?;

        // Schema --Use--> SchemaVariant (this)
        Schema::add_edge_to_variant(ctx, schema_id, id.into(), EdgeWeightKind::new_use()).await?;

        let schema_variant_id: SchemaVariantId = id.into();
        let root_prop = RootProp::new(ctx, schema_variant_id).await?;
        let _func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;

        let schema_variant = Self::assemble(id.into(), content);
        Ok((schema_variant, root_prop))
    }

    pub async fn dump_props_as_list(&self, ctx: &DalContext) -> SchemaVariantResult<Vec<PropPath>> {
        let mut props = vec![];

        let root_prop_id = Self::get_root_prop_id(ctx, self.id()).await?;
        let mut work_queue = VecDeque::from([(root_prop_id, None::<PropPath>)]);
        let workspace_snapshot = ctx.workspace_snapshot()?;

        while let Some((prop_id, maybe_parent_path)) = work_queue.pop_front() {
            let node_weight = workspace_snapshot.get_node_weight_by_id(prop_id).await?;

            match node_weight {
                NodeWeight::Prop(prop_inner) => {
                    let name = prop_inner.name();

                    let path = match &maybe_parent_path {
                        Some(parent_path) => parent_path.join(&PropPath::new([name])),
                        None => PropPath::new([name]),
                    };

                    props.push(path.clone());

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
                            work_queue.push_back((id.into(), Some(path.clone())));
                        }
                    }
                }
                _ => return Err(SchemaVariantError::PropIdNotAProp(prop_id)),
            }
        }

        Ok(props)
    }

    pub async fn get_by_id(ctx: &DalContext, id: SchemaVariantId) -> SchemaVariantResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: SchemaVariantContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here
        let SchemaVariantContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
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
    pub async fn list_ids(ctx: &DalContext) -> SchemaVariantResult<Vec<SchemaVariantId>> {
        let schema_ids = Schema::list_ids(ctx).await?;

        let mut schema_variant_ids = Vec::new();

        let workspace_snapshot = ctx.workspace_snapshot()?;
        for schema_id in schema_ids {
            let schema_variant_node_indices = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    schema_id,
                    EdgeWeightKind::new_use_default().into(),
                )
                .await?;

            for schema_variant_node_index in schema_variant_node_indices {
                let raw_id = workspace_snapshot
                    .get_node_weight(schema_variant_node_index)
                    .await?
                    .id();
                schema_variant_ids.push(raw_id.into());
            }
        }

        Ok(schema_variant_ids)
    }

    pub async fn get_default_for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<Self> {
        let default_schema_variant_id = Schema::get_default_schema_variant_by_id(ctx, schema_id)
            .await?
            .ok_or(SchemaVariantError::DefaultSchemaVariantNotFound(schema_id))?;

        Self::get_by_id(ctx, default_schema_variant_id).await
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

        let node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind_by_index(
                parent_index,
                EdgeWeightKind::new_use().into(),
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
                Some(func_content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let SchemaVariantContent::V1(inner) = func_content;

                    schema_variants.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        // Now add the default schema variant to the list as this is a different edge kind
        let default_schema_variant = Self::get_default_for_schema(ctx, schema_id).await?;
        schema_variants.push(default_schema_variant);

        Ok(schema_variants)
    }

    pub fn id(&self) -> SchemaVariantId {
        self.id
    }

    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn display_name(&self) -> Option<String> {
        self.display_name.clone()
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
        destination_id: ActionPrototypeId,
        add_fn: add_edge_to_deprecated_action_prototype,
        discriminant: EdgeWeightKindDiscriminants::ActionPrototype,
        result: SchemaVariantResult,
    );
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
        add_fn: add_edge_to_deprecated_action_func,
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

    pub async fn new_action_prototype(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        Self::add_edge_to_deprecated_action_func(
            ctx,
            schema_variant_id,
            func_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        Ok(())
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

    #[allow(dead_code)]
    async fn get_content(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<(ContentHash, SchemaVariantContentV1)> {
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

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let SchemaVariantContent::V1(inner) = content;

        Ok((hash, inner))
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
        //Self::mark_props_as_able_to_be_used_as_prototype_args(ctx, schema_variant_id)?;

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
        component_id: Option<ComponentId>,
        leaf_kind: LeafKind,
        input_locations: &[LeafInputLocation],
        func: &Func,
    ) -> SchemaVariantResult<AttributePrototypeId> {
        let leaf_item_prop_id =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

        if component_id.is_some() {
            unimplemented!("component context not supported for leaf functions");
        }

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

        Ok(
            match AttributePrototype::find_for_prop(ctx, leaf_item_prop_id, &key).await? {
                Some(existing_proto_id) => {
                    let apas =
                        AttributePrototypeArgument::list_ids_for_prototype(ctx, existing_proto_id)
                            .await?;

                    let mut apa_func_arg_ids = HashMap::new();
                    for input in &inputs {
                        let mut exisiting_func_arg = None;
                        for apa_id in &apas {
                            let func_arg_id =
                                AttributePrototypeArgument::func_argument_id_by_id(ctx, *apa_id)
                                    .await?;
                            apa_func_arg_ids.insert(apa_id, func_arg_id);

                            if func_arg_id == input.func_argument_id {
                                exisiting_func_arg = Some(func_arg_id);
                            }
                        }

                        if exisiting_func_arg.is_none() {
                            let input_prop_id = Self::find_root_child_prop_id(
                                ctx,
                                schema_variant_id,
                                input.location.into(),
                            )
                            .await?;

                            info!(
                                "adding root child func arg: {:?}, {:?}",
                                input_prop_id, input.location
                            );

                            let new_apa = AttributePrototypeArgument::new(
                                ctx,
                                existing_proto_id,
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

                    existing_proto_id
                }
                None => {
                    let (_, new_proto) = SchemaVariant::add_leaf(
                        ctx,
                        func.id,
                        schema_variant_id,
                        component_id,
                        leaf_kind,
                        inputs,
                    )
                    .await?;

                    new_proto
                }
            },
        )
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

    pub async fn schema(&self, ctx: &DalContext) -> SchemaVariantResult<Schema> {
        Self::schema_for_schema_variant_id(ctx, self.id).await
    }

    pub async fn schema_for_schema_variant_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Schema> {
        let schema_id = {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            let maybe_schema_indices = workspace_snapshot
                .edges_directed(schema_variant_id, Incoming)
                .await?;

            let mut schema_id: Option<SchemaId> = None;
            for (edge_weight, source_index, _) in maybe_schema_indices {
                if *edge_weight.kind() == EdgeWeightKind::new_use_default() {
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

        Ok(Schema::get_by_id(ctx, schema_id).await?)
    }

    pub async fn all_funcs(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<Func>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut all_funcs = vec![];

        let sv = Self::get_by_id(ctx, schema_variant_id).await?;

        let prop_list = sv.dump_props_as_list(ctx).await?;
        for prop_path in prop_list {
            let prop_id = Prop::find_prop_id_by_path(ctx, schema_variant_id, &prop_path).await?;
            // Let's get the Attribute funcs now
            if let Some(ap_id) = AttributePrototype::find_for_prop(ctx, prop_id, &None).await? {
                let func_id = AttributePrototype::func_id(ctx, ap_id).await?;

                let node_weight = workspace_snapshot
                    .get_node_weight_by_id(func_id)
                    .await?
                    .get_func_node_weight()?;

                if node_weight.func_kind() == FuncKind::Attribute {
                    let func = Func::get_by_id(ctx, func_id).await?;
                    all_funcs.push(func);
                }
            }

            // Now let's get all of the outgoing edges for the Prop
            for (edge_weight, _source, _target) in
                workspace_snapshot.edges_directed(prop_id, Outgoing).await?
            {
                if let EdgeWeightKind::Prototype(Some(key)) = edge_weight.kind() {
                    if let Some(func_id) = Func::find_by_name(ctx, key).await? {
                        let func = Func::get_by_id(ctx, func_id).await?;
                        all_funcs.push(func);
                    }
                }
            }
        }

        // Let's get all of the Authentication funcs
        let auth_func_ids =
            Self::list_auth_func_ids_for_schema_variant(ctx, schema_variant_id).await?;
        for auth_func_id in auth_func_ids {
            let auth_func = Func::get_by_id(ctx, auth_func_id).await?;
            // We may not need this - the list_auth_func_ids_for_schema_variant returns multiple
            // of the same type
            if !all_funcs.contains(&auth_func) {
                all_funcs.push(auth_func);
            }
        }

        let action_prototype_nodes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::ActionPrototype,
            )
            .await?;
        for action_prototype_node in action_prototype_nodes {
            let weight = workspace_snapshot
                .get_node_weight(action_prototype_node)
                .await?;
            let ap = ActionPrototype::get_by_id(ctx, weight.id().into())
                .await
                .map_err(|e| SchemaVariantError::ActionPrototype(e.to_string()))?;
            let func = Func::get_by_id(
                ctx,
                ap.func_id(ctx)
                    .await
                    .map_err(|e| SchemaVariantError::ActionPrototype(e.to_string()))?,
            )
            .await?;
            all_funcs.push(func);
        }

        Ok(all_funcs)
    }

    pub async fn list_auth_func_ids_for_schema_variant(
        ctx: &DalContext,
        variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Vec<FuncId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut auth_funcs = vec![];

        for node_id in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                variant_id,
                EdgeWeightKindDiscriminants::AuthenticationPrototype,
            )
            .await?
        {
            auth_funcs.push(
                workspace_snapshot
                    .get_node_weight(node_id)
                    .await?
                    .id()
                    .into(),
            )
        }

        Ok(auth_funcs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantMetadataJson {
    /// Name for this variant. Actually, this is the name for this [`Schema`](crate::Schema), we're
    /// punting on the issue of multiple variants for the moment.
    pub name: String,
    /// Override for the UI name for this schema
    #[serde(alias = "menu_name")]
    pub menu_name: Option<String>,
    /// The category this schema variant belongs to
    pub category: String,
    /// The color for the component on the component diagram as a hex string
    pub color: String,
    #[serde(alias = "component_type")]
    pub component_type: ComponentType,
    pub link: Option<String>,
    pub description: Option<String>,
}

impl SchemaVariantMetadataJson {
    pub fn to_spec(&self, variant: SchemaVariantSpec) -> SchemaVariantResult<SchemaSpec> {
        let mut builder = SchemaSpec::builder();
        builder.name(&self.name);
        let mut data_builder = SchemaSpecData::builder();
        data_builder.name(&self.name);
        data_builder.category(&self.category);
        if let Some(menu_name) = &self.menu_name {
            data_builder.category_name(menu_name.as_str());
        }
        builder.data(data_builder.build()?);
        builder.variant(variant);

        Ok(builder.build()?)
    }
}

/// The json definition for a [`SchemaVariant`](crate::SchemaVariant)'s [`Prop`](crate::Prop) tree (and
/// more in the future).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantJson {
    /// The immediate child [`Props`](crate::Prop) underneath "/root/domain".
    #[serde(default)]
    pub props: Vec<PropDefinition>,
    /// The immediate child [`Props`](crate::Prop) underneath "/root/secrets".
    #[serde(default)]
    pub secret_props: Vec<PropDefinition>,
    /// The immediate child [`Props`](crate::Prop) underneath "/root/secretsDefinition".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_definition: Option<Vec<PropDefinition>>,
    /// The immediate child [`Props`](crate::Prop) underneath "/root/resource_value".
    #[serde(default)]
    pub resource_props: Vec<PropDefinition>,
    /// Identity relationships for [`Props`](crate::Prop) underneath "/root/si".
    #[serde(default)]
    pub si_prop_value_froms: Vec<SiPropValueFrom>,

    /// The input [`Sockets`](crate::Socket) and created for the [`variant`](crate::SchemaVariant).
    #[serde(default)]
    pub input_sockets: Vec<SocketDefinition>,
    /// The output [`Sockets`](crate::Socket) and created for the [`variant`](crate::SchemaVariant).
    #[serde(default)]
    pub output_sockets: Vec<SocketDefinition>,
    /// A map of documentation links to reference. To reference links (values) specify the key via
    /// the "doc_link_ref" field for a [`PropDefinition`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_links: Option<HashMap<String, String>>,
}

impl SchemaVariantJson {
    pub fn to_spec(
        &self,
        metadata: SchemaVariantMetadataJson,
        identity_func_unique_id: &str,
        asset_func_spec_unique_id: &str,
    ) -> SchemaVariantResult<SchemaVariantSpec> {
        let mut builder = SchemaVariantSpec::builder();
        let name = "v0";
        builder.name(name);

        let mut data_builder = SchemaVariantSpecData::builder();

        data_builder.name(name);
        data_builder.color(metadata.color);
        data_builder.component_type(metadata.component_type);
        if let Some(link) = metadata.link {
            data_builder.try_link(link.as_str())?;
        }

        data_builder.func_unique_id(asset_func_spec_unique_id);
        builder.data(data_builder.build()?);

        for si_prop_value_from in &self.si_prop_value_froms {
            builder.si_prop_func(si_prop_value_from.to_spec(identity_func_unique_id));
        }
        for prop in &self.props {
            builder.domain_prop(prop.to_spec(identity_func_unique_id)?);
        }
        for prop in &self.secret_props {
            builder.secret_prop(prop.to_spec(identity_func_unique_id)?);
        }
        if let Some(props) = &self.secret_definition {
            for prop in props {
                builder.secret_definition_prop(prop.to_spec(identity_func_unique_id)?);
            }
        }
        for resource_prop in &self.resource_props {
            builder.resource_value_prop(resource_prop.to_spec(identity_func_unique_id)?);
        }
        for input_socket in &self.input_sockets {
            builder.socket(input_socket.to_spec(true, identity_func_unique_id)?);
        }
        for output_socket in &self.output_sockets {
            builder.socket(output_socket.to_spec(false, identity_func_unique_id)?);
        }

        Ok(builder.build()?)
    }

    pub fn metadata_from_spec(
        schema_spec: SchemaSpec,
    ) -> SchemaVariantResult<SchemaVariantMetadataJson> {
        let schema_data = schema_spec.data.unwrap_or(SchemaSpecData {
            name: schema_spec.name.to_owned(),
            default_schema_variant: None,
            category: "".into(),
            category_name: None,
            ui_hidden: false,
        });

        let default_variant_spec = match schema_data.default_schema_variant {
            Some(default_variant_unique_id) => schema_spec
                .variants
                .iter()
                .find(|variant| variant.unique_id.as_deref() == Some(&default_variant_unique_id))
                .ok_or(SchemaVariantError::DefaultVariantNotFound(
                    default_variant_unique_id,
                ))?,
            None => schema_spec
                .variants
                .last()
                .ok_or(SchemaVariantError::NoVariants)?,
        };

        let variant_spec_data =
            default_variant_spec
                .data
                .to_owned()
                .unwrap_or(SchemaVariantSpecData {
                    name: "v0".into(),
                    color: None,
                    link: None,
                    component_type: si_pkg::SchemaVariantSpecComponentType::Component,
                    func_unique_id: "0".into(),
                });

        let metadata = SchemaVariantMetadataJson {
            name: schema_spec.name,
            menu_name: schema_data.category_name,
            category: schema_data.category,
            color: variant_spec_data
                .color
                .to_owned()
                .unwrap_or(DEFAULT_SCHEMA_VARIANT_COLOR.into()),
            component_type: variant_spec_data.component_type.into(),
            link: variant_spec_data.link.as_ref().map(|l| l.to_string()),
            description: None, // XXX - does this exist?
        };

        Ok(metadata)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropWidgetDefinition {
    /// The [`kind`](crate::property_editor::schema::WidgetKind) of the [`Prop`](crate::Prop) to be created.
    kind: WidgetKind,
    /// The `Option<Value>` of the [`kind`](crate::property_editor::schema::WidgetKind) to be created.
    #[serde(default)]
    options: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapKeyFunc {
    pub key: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_from: Option<ValueFrom>,
}

impl MapKeyFunc {
    pub fn to_spec(&self, identity_func_unique_id: &str) -> SchemaVariantResult<MapKeyFuncSpec> {
        let mut builder = MapKeyFuncSpec::builder();
        builder.func_unique_id(identity_func_unique_id);
        builder.key(&self.key);
        if let Some(value_from) = &self.value_from {
            builder.input(value_from.to_spec());
        };
        Ok(builder.build()?)
    }
}

/// The definition for a [`Prop`](crate::Prop) in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropDefinition {
    /// The name of the [`Prop`](crate::Prop) to be created.
    pub name: String,
    /// The [`kind`](crate::PropKind) of the [`Prop`](crate::Prop) to be created.
    pub kind: PropKind,
    /// An optional reference to a documentation link in the "doc_links" field for the
    /// [`SchemaVariantJson`] for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_link_ref: Option<String>,
    /// An optional documentation link for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_link: Option<String>,
    /// An optional set of inline documentation for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// If our [`kind`](crate::PropKind) is [`Object`](crate::PropKind::Object), specify the
    /// child definition(s).
    #[serde(default)]
    pub children: Vec<PropDefinition>,
    /// If our [`kind`](crate::PropKind) is [`Array`](crate::PropKind::Array), specify the entry
    /// definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Box<PropDefinition>>,
    /// The [`WidgetDefinition`](crate::schema::variant::definition::PropWidgetDefinition) of the
    /// [`Prop`](crate::Prop) to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget: Option<PropWidgetDefinition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    // The source of the information for the prop
    pub value_from: Option<ValueFrom>,
    // Whether the prop is hidden from the UI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_key_funcs: Option<Vec<MapKeyFunc>>,
}

impl PropDefinition {
    pub fn to_spec(&self, identity_func_unique_id: &str) -> SchemaVariantResult<PropSpec> {
        let mut builder = PropSpec::builder();
        builder.name(&self.name);
        builder.kind(self.kind);
        builder.has_data(true);
        if let Some(doc_url) = &self.doc_link {
            builder.try_doc_link(doc_url.as_str())?;
        }
        if let Some(docs) = &self.documentation {
            builder.documentation(docs);
        }
        if let Some(default_value) = &self.default_value {
            builder.default_value(default_value.to_owned());
        }
        match self.kind {
            PropKind::Array | PropKind::Map => {
                if let Some(entry) = &self.entry {
                    builder.type_prop(entry.to_spec(identity_func_unique_id)?);
                }
            }
            PropKind::Object => {
                for child in &self.children {
                    builder.entry(child.to_spec(identity_func_unique_id)?);
                }
            }
            _ => {}
        }
        if let Some(widget) = &self.widget {
            builder.widget_kind(widget.kind);
            if let Some(widget_options) = &widget.options {
                builder.widget_options(widget_options.to_owned());
            }
        }
        if let Some(value_from) = &self.value_from {
            builder.func_unique_id(identity_func_unique_id);
            builder.input(value_from.to_spec());
        }
        if let Some(hidden) = self.hidden {
            builder.hidden(hidden);
        }
        if let Some(map_key_funcs) = &self.map_key_funcs {
            for map_key_func in map_key_funcs {
                builder.map_key_func(map_key_func.to_spec(identity_func_unique_id)?);
            }
        }

        Ok(builder.build()?)
    }
}

/// The definition for a [`Socket`](crate::Socket) in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketDefinition {
    /// The name of the [`Socket`](crate::Socket) to be created.
    pub name: String,
    /// The type identifier of the [`Socket`](crate::Socket) to be created.
    pub connection_annotations: String,
    /// The [`arity`](https://en.wikipedia.org/wiki/Arity) of the [`Socket`](crate::Socket).
    /// Defaults to [`SocketArity::Many`](crate::SocketArity::Many) if nothing is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arity: Option<SocketArity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_hidden: Option<bool>,
    // The source of the information for the socket
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_from: Option<ValueFrom>,
}

impl SocketDefinition {
    pub fn to_spec(
        &self,
        is_input: bool,
        identity_func_unique_id: &str,
    ) -> SchemaVariantResult<SocketSpec> {
        let mut builder = SocketSpec::builder();
        let mut data_builder = SocketSpecData::builder();
        builder.name(&self.name);
        data_builder.name(&self.name);
        data_builder.connection_annotations(&self.connection_annotations);
        if is_input {
            data_builder.kind(SocketSpecKind::Input);
        } else {
            data_builder.kind(SocketSpecKind::Output);
        }

        if let Some(arity) = &self.arity {
            data_builder.arity(arity);
        } else {
            data_builder.arity(SocketSpecArity::Many);
        }
        if let Some(hidden) = &self.ui_hidden {
            data_builder.ui_hidden(*hidden);
        } else {
            data_builder.ui_hidden(false);
        }
        if let Some(value_from) = &self.value_from {
            data_builder.func_unique_id(identity_func_unique_id);
            builder.input(value_from.to_spec());
        }
        builder.data(data_builder.build()?);

        Ok(builder.build()?)
    }
}

/// The definition for the source of the information for a prop or a socket in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ValueFrom {
    InputSocket { socket_name: String },
    OutputSocket { socket_name: String },
    Prop { prop_path: Vec<String> },
}

impl ValueFrom {
    fn to_spec(&self) -> AttrFuncInputSpec {
        match self {
            ValueFrom::InputSocket { socket_name } => AttrFuncInputSpec::InputSocket {
                name: "identity".to_string(),
                socket_name: socket_name.to_owned(),
                unique_id: None,
                deleted: false,
            },
            ValueFrom::Prop { prop_path } => AttrFuncInputSpec::Prop {
                name: "identity".to_string(),
                prop_path: PropPath::new(prop_path).into(),
                unique_id: None,
                deleted: false,
            },
            ValueFrom::OutputSocket { socket_name } => AttrFuncInputSpec::OutputSocket {
                name: "identity".to_string(),
                socket_name: socket_name.to_owned(),
                unique_id: None,
                deleted: false,
            },
        }
    }
}

/// The definition for the source of the data for prop under "/root/"si" in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiPropValueFrom {
    kind: SiPropFuncSpecKind,
    value_from: ValueFrom,
}

impl SiPropValueFrom {
    fn to_spec(&self, identity_func_unique_id: &str) -> SiPropFuncSpec {
        SiPropFuncSpec {
            kind: self.kind,
            func_unique_id: identity_func_unique_id.to_owned(),
            inputs: vec![self.value_from.to_spec()],
            unique_id: None,
            deleted: false,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantMetadataView {
    id: SchemaVariantId,
    name: String,
    category: String,
    #[serde(alias = "display_name")]
    display_name: Option<String>,
    color: String,
    component_type: ComponentType,
    link: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl SchemaVariantMetadataView {
    pub async fn list(ctx: &DalContext) -> SchemaVariantResult<Vec<Self>> {
        let mut views = Vec::new();

        let schemas = Schema::list(ctx).await?;
        for schema in schemas {
            let default_schema_variant =
                SchemaVariant::get_default_for_schema(ctx, schema.id()).await?;
            views.push(SchemaVariantMetadataView {
                id: default_schema_variant.id,
                name: schema.name.to_owned(),
                category: default_schema_variant.category.to_owned(),
                color: default_schema_variant.get_color(ctx).await?,
                timestamp: default_schema_variant.timestamp.to_owned(),
                component_type: default_schema_variant
                    .get_type(ctx)
                    .await?
                    .unwrap_or(ComponentType::Component),
                link: default_schema_variant.link.to_owned(),
                description: default_schema_variant.description,
                display_name: default_schema_variant.display_name,
            })
        }

        Ok(views)
    }
}
