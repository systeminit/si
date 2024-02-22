//! This module contains [`SchemaVariant`](crate::SchemaVariant), which is t/he "class" of a
//! [`Component`](crate::Component).

use content_store::{ContentHash, Store};
use petgraph::prelude::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::FuncError;
use crate::prop::{PropError, PropPath};
use crate::provider::external::{ExternalProviderContent, ExternalProviderError};
use crate::provider::internal::{InternalProviderContent, InternalProviderError};
use crate::schema::variant::root_prop::RootProp;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::NodeIndex;

use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError, PropNodeWeight};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk,
    schema::variant::leaves::{LeafInput, LeafInputLocation, LeafKind},
    AttributePrototype, AttributePrototypeId, ComponentId, DalContext, ExternalProvider,
    ExternalProviderId, Func, FuncId, InternalProvider, Prop, PropId, PropKind, Schema,
    SchemaError, SchemaId, Timestamp, TransactionsError,
};
use crate::{FuncBackendResponseType, InternalProviderId};

use self::root_prop::RootPropChild;

// use self::leaves::{LeafInput, LeafInputLocation, LeafKind};

pub mod definition;
pub mod leaves;
pub mod root_prop;

// FIXME(nick,theo): colors should be required for all schema variants.
// There should be no default in the backend as there should always be a color.
pub const DEFAULT_SCHEMA_VARIANT_COLOR: &str = "00b0bc";
pub const SCHEMA_VARIANT_VERSION: SchemaVariantContentDiscriminants =
    SchemaVariantContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute argument prototype error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("Func {0} of response type {1} cannot set leaf {2:?}")]
    LeafFunctionMismatch(FuncId, FuncBackendResponseType, LeafKind),
    #[error("func {0} not a JsAttribute func, required for leaf functions")]
    LeafFunctionMustBeJsAttribute(FuncId),
    #[error("Leaf map prop not found for item prop {0}")]
    LeafMapPropNotFound(PropId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
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
    #[error("store error: {0}")]
    Store(#[from] content_store::StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
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
    // The [`RootProp`](crate::RootProp) for [`self`](Self).
    //root_prop_id: Option<PropId>,
    // schema_variant_definition_id: Option<SchemaVariantDefinitionId>,
    link: Option<String>,
    finalized_once: bool,
    category: String,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SchemaVariantContent {
    V1(SchemaVariantContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaVariantContentV1 {
    pub timestamp: Timestamp,
    pub ui_hidden: bool,
    pub name: String,
    // The [`RootProp`](crate::RootProp) for [`self`](Self).
    // pub root_prop_id: Option<PropId>,
    // pub schema_variant_definition_id: Option<SchemaVariantDefinitionId>,
    pub link: Option<String>,
    pub finalized_once: bool,
    pub category: String,
}

impl SchemaVariant {
    pub fn assemble(id: SchemaVariantId, inner: SchemaVariantContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            link: inner.link,
            ui_hidden: inner.ui_hidden,
            finalized_once: inner.finalized_once,
            category: inner.category,
        }
    }

    pub async fn new(
        ctx: &DalContext,
        schema_id: SchemaId,
        name: impl Into<String>,
        category: impl Into<String>,
    ) -> SchemaVariantResult<(Self, RootProp)> {
        info!("creating schema variant and root prop tree");
        let content = SchemaVariantContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            link: None,
            ui_hidden: false,
            finalized_once: false,
            category: category.into(),
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&SchemaVariantContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            let node_weight =
                NodeWeight::new_content(change_set, id, ContentAddress::SchemaVariant(hash))?;
            let _node_index = workspace_snapshot.add_node(node_weight)?;

            // Schema --Use--> SchemaVariant (this)
            workspace_snapshot.add_edge(
                schema_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                id,
            )?;
        }

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

        while let Some((prop_id, maybe_parent_path)) = work_queue.pop_front() {
            let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
            let node_weight = workspace_snapshot.get_node_weight_by_id(prop_id)?;

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
                        )?
                        .get(0)
                    {
                        let ordering_node_weight = workspace_snapshot
                            .get_node_weight(*ordering_node_idx)?
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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: SchemaVariantContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here
        let SchemaVariantContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
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

    pub async fn list_for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let mut schema_variants = vec![];
        let parent_index = workspace_snapshot.get_node_index_by_id(schema_id)?;

        let node_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind_by_index(
            parent_index,
            EdgeWeightKindDiscriminants::Use,
        )?;

        let mut node_weights = vec![];
        let mut content_hashes = vec![];
        for index in node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::SchemaVariant)?;
            content_hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let content_map: HashMap<ContentHash, SchemaVariantContent> = ctx
            .content_store()
            .lock()
            .await
            .get_bulk(content_hashes.as_slice())
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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let edge_targets: Vec<NodeIndex> = workspace_snapshot
            .edges_directed(schema_variant_id, Direction::Outgoing)?
            .map(|edge_ref| edge_ref.target())
            .collect();

        for index in edge_targets {
            let node_weight = workspace_snapshot.get_node_weight(index)?;
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
        info!("creating default prototypes");
        let change_set = ctx.change_set_pointer()?;
        let func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;
        let root_prop_node_weight = Self::get_root_prop_node_weight(ctx, schema_variant_id).await?;
        let mut work_queue: VecDeque<PropNodeWeight> = VecDeque::from(vec![root_prop_node_weight]);

        while let Some(prop) = work_queue.pop_front() {
            // See an attribute prototype exists.
            let mut found_attribute_prototype_id: Option<AttributePrototypeId> = None;
            {
                let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
                let targets = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
                    prop.id(),
                    EdgeWeightKindDiscriminants::Prototype,
                )?;
                for target in targets {
                    let node_weight = workspace_snapshot.get_node_weight(target)?;
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
                let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
                workspace_snapshot.add_edge(
                    prop.id(),
                    EdgeWeight::new(change_set, EdgeWeightKind::Prototype(None))?,
                    attribute_prototype.id(),
                )?;
            }

            // Push all children onto the work queue.
            let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
            let targets = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
                prop.id(),
                EdgeWeightKindDiscriminants::Use,
            )?;
            for target in targets {
                let node_weight = workspace_snapshot.get_node_weight(target)?;
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
        let root_prop_node_weight = Self::get_root_prop_node_weight(ctx, schema_variant_id).await?;
        let root_prop_idx = {
            let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
            workspace_snapshot.get_node_index_by_id(root_prop_node_weight.id())?
        };

        let mut work_queue = VecDeque::new();
        work_queue.push_back(root_prop_idx);

        while let Some(prop_idx) = work_queue.pop_front() {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            workspace_snapshot.mark_prop_as_able_to_be_used_as_prototype_arg(prop_idx)?;

            let node_weight = workspace_snapshot.get_node_weight(prop_idx)?.to_owned();
            if let NodeWeight::Prop(prop) = node_weight {
                // Only descend if we are an object.
                if prop.kind() == PropKind::Object {
                    let targets = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
                        prop.id(),
                        EdgeWeightKindDiscriminants::Use,
                    )?;
                    work_queue.extend(targets);
                }
            }
        }

        Ok(())
    }

    pub async fn new_action_prototype(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
        workspace_snapshot.add_edge(
            schema_variant_id,
            EdgeWeight::new(ctx.change_set_pointer()?, EdgeWeightKind::Use)?,
            func_id,
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn get_content(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<(ContentHash, SchemaVariantContentV1)> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let id: Ulid = schema_variant_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: SchemaVariantContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let SchemaVariantContent::V1(inner) = content;

        Ok((hash, inner))
    }

    /// This _idempotent_ function "finalizes" a [`SchemaVariant`].
    ///
    /// Once a [`SchemaVariant`] has had all of its [`Props`](crate::Prop) created, there are a few
    /// things that need to happen before it is usable:
    ///
    /// * Create the default [`AttributePrototypes`](crate::AttributePrototype)
    /// * Create the _internally consuming_ [`InternalProviders`](crate::InternalProvider)
    ///   corresponding to every [`Prop`](crate::Prop) in the [`SchemaVariant`] that is not a
    ///   descendant of an Array or a Map.
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

    pub async fn get_color(&self, ctx: &DalContext) -> SchemaVariantResult<Option<String>> {
        let color_prop_id =
            Prop::find_prop_id_by_path(ctx, self.id, &PropPath::new(["root", "si", "color"]))
                .await?;

        let prototype_id = Prop::prototype_id(ctx, color_prop_id).await?;

        match AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id)
            .await?
            .get(0)
        {
            None => Ok(None),
            Some(apa_id) => {
                match AttributePrototypeArgument::static_value_by_id(ctx, *apa_id).await? {
                    Some(static_value) => {
                        let color: String = serde_json::from_value(static_value.value)?;
                        Ok(Some(color))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    /// Configures the "default" value for the
    /// [`AttributePrototypeArgument`](crate::attribute::prototype::argument::AttributePrototypeArgument)
    /// for the /root/si/color [`Prop`](crate::Prop). If a prototype already
    /// exists pointing to a function other than
    /// [`IntrinsicFunc::SetString`](`crate::func::intrinsics::IntrinsicFunc::SetString`)
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
    /// [`IntrinsicFunc::SetString`](`crate::func::intrinsics::IntrinsicFunc::SetString`)
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

    pub async fn list_external_providers_and_explicit_internal_providers(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<(Vec<ExternalProvider>, Vec<InternalProvider>)> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        // Look for all external and explicit internal providers that the schema variant uses.
        let maybe_provider_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_variant_id,
            EdgeWeightKindDiscriminants::Provider,
        )?;

        // Collect the external and the explicit internal providers separately.
        let mut external_provider_hashes: Vec<(ExternalProviderId, ContentHash)> = Vec::new();
        let mut explicit_internal_provider_hashes: Vec<(InternalProviderId, ContentHash)> =
            Vec::new();

        for maybe_provider_index in maybe_provider_indices {
            let node_weight = workspace_snapshot.get_node_weight(maybe_provider_index)?;
            if let NodeWeight::Content(content_node_weight) = node_weight {
                match content_node_weight.content_address() {
                    ContentAddress::ExternalProvider(external_provider_content_hash) => {
                        external_provider_hashes.push((
                            content_node_weight.id().into(),
                            external_provider_content_hash,
                        ));
                    }
                    ContentAddress::InternalProvider(internal_provider_content_hash) => {
                        explicit_internal_provider_hashes.push((
                            content_node_weight.id().into(),
                            internal_provider_content_hash,
                        ));
                    }
                    _ => {}
                }
            }
        }

        // Grab all the contents in bulk from the content store.
        let external_provider_hashes_only: Vec<ContentHash> =
            external_provider_hashes.iter().map(|(_, h)| *h).collect();
        let external_provider_content_map: HashMap<ContentHash, ExternalProviderContent> = ctx
            .content_store()
            .lock()
            .await
            .get_bulk(external_provider_hashes_only.as_slice())
            .await?;
        let explicit_internal_provider_hashes_only: Vec<ContentHash> =
            explicit_internal_provider_hashes
                .iter()
                .map(|(_, h)| *h)
                .collect();
        let internal_provider_content_map: HashMap<ContentHash, InternalProviderContent> = ctx
            .content_store()
            .lock()
            .await
            .get_bulk(explicit_internal_provider_hashes_only.as_slice())
            .await?;

        // Assemble all external providers.
        let mut external_providers = Vec::with_capacity(external_provider_hashes.len());
        for (external_provider_id, external_provider_hash) in external_provider_hashes {
            let external_provider_content = external_provider_content_map
                .get(&external_provider_hash)
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    external_provider_id.into(),
                ))?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let ExternalProviderContent::V1(external_provider_content_inner) =
                external_provider_content;

            external_providers.push(ExternalProvider::assemble(
                external_provider_id,
                external_provider_content_inner.to_owned(),
            ));
        }

        // Assemble all explicit internal providers.
        let mut explicit_internal_providers =
            Vec::with_capacity(explicit_internal_provider_hashes.len());
        for (internal_provider_id, internal_provider_hash) in explicit_internal_provider_hashes {
            let internal_provider_content = internal_provider_content_map
                .get(&internal_provider_hash)
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    internal_provider_id.into(),
                ))?;

            // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
            let InternalProviderContent::V1(internal_provider_content_inner) =
                internal_provider_content;

            explicit_internal_providers.push(InternalProvider::assemble(
                internal_provider_id,
                internal_provider_content_inner.to_owned(),
            ));
        }

        Ok((external_providers, explicit_internal_providers))
    }

    pub async fn schema(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Schema> {
        let schema_id = {
            let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
            let maybe_schema_indices = workspace_snapshot.incoming_sources_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Use,
            )?;

            let mut schema_id: Option<SchemaId> = None;
            for index in maybe_schema_indices {
                if let NodeWeight::Content(content) = workspace_snapshot.get_node_weight(index)? {
                    let content_hash_discriminants: ContentAddressDiscriminants =
                        content.content_address().into();
                    if let ContentAddressDiscriminants::Schema = content_hash_discriminants {
                        // TODO(nick): consider creating a new edge weight kind to make this easier.
                        // We also should use a proper error here.
                        schema_id = match schema_id {
                            None => Some(content.id().into()),
                            Some(_already_found_schema_id) => {
                                panic!("already found a schema")
                            }
                        };
                    }
                }
            }
            schema_id.ok_or(SchemaVariantError::SchemaNotFound(schema_variant_id))?
        };

        Ok(Schema::get_by_id(ctx, schema_id).await?)
    }
}
