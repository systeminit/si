use std::{
    collections::BTreeMap,
    sync::Arc,
};

use serde_json::json;
use si_id::{
    AttributeValueId,
    FuncId,
    LeafPrototypeId,
    SchemaId,
    ulid::Ulid,
};
use si_layer_cache::LayerDbError;
use thiserror::Error;
use tokio::sync::RwLock;

use super::{
    Schema,
    SchemaError,
};
use crate::{
    AttributeValue,
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    Func,
    FuncError,
    HelperError,
    WorkspaceSnapshotError,
    attribute::{
        path::AttributePath,
        value::{
            AttributeValueError,
            PrototypeExecution,
            write_values_to_cas,
        },
    },
    func::{
        leaf::{
            LeafInputLocation,
            LeafKind,
        },
        runner::{
            FuncRunner,
            FuncRunnerError,
        },
    },
    implement_add_edge_to,
    layer_db_types::{
        AttributePathsContent,
        AttributePathsContentV1,
        ContentTypes,
    },
    workspace_snapshot::{
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        node_weight::{
            NodeWeight,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
            leaf_prototype_node_weight::LeafPrototypeNodeWeight,
        },
    },
};

#[derive(Error, Debug)]
pub enum LeafPrototypeError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("component with root av id {0} is missing a a leaf destination at {1}")]
    ComponentMissingDestinationAv(AttributeValueId, String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] FuncRunnerError),
    #[error("Helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("leaf prototype {0} has no function")]
    LeafPrototypeHasNoFunc(LeafPrototypeId),
    #[error("leaf prototype has no inputs at content address: {0:?}")]
    LeafPrototypeHasNoInputsAtContentAddress(ContentAddress),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshotError(#[from] WorkspaceSnapshotError),
}

pub type LeafPrototypeResult<T> = Result<T, LeafPrototypeError>;

/// Leaf prototypes are schema level ("overlay") equivalents of Attribute
/// functions, with the exception that they have predefined output locations
/// (defined by their "LeafKind"). Currently there are only Qualifications and
/// CodeGenerations.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LeafPrototype {
    id: LeafPrototypeId,
    inputs: Vec<AttributePath>,
    kind: LeafKind,
}

impl LeafPrototype {
    pub fn id(&self) -> LeafPrototypeId {
        self.id
    }

    pub fn inputs(&self) -> &[AttributePath] {
        &self.inputs
    }

    pub fn leaf_inputs(&self) -> impl Iterator<Item = LeafInputLocation> {
        self.inputs().iter().filter_map(|path| path.into())
    }

    pub fn kind(&self) -> LeafKind {
        self.kind
    }

    pub async fn new(
        ctx: &DalContext,
        schema_id: SchemaId,
        kind: LeafKind,
        inputs: &[LeafInputLocation],
        func_id: FuncId,
    ) -> LeafPrototypeResult<Self> {
        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;

        let attribute_paths: Vec<AttributePath> = inputs.iter().copied().map(Into::into).collect();

        let (content_hash, _) = ctx.layer_db().cas().write(
            Arc::new(crate::layer_db_types::ContentTypes::AttributePaths(
                attribute_paths.clone().into(),
            )),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let node_weight = NodeWeight::LeafPrototype(LeafPrototypeNodeWeight::new(
            id,
            lineage_id,
            kind,
            content_hash,
        ));

        let snap = ctx.workspace_snapshot()?;
        snap.add_or_replace_node(node_weight).await?;

        let leaf_prototype_id = id.into();
        Schema::add_edge_to_leaf_prototype(
            ctx,
            schema_id,
            leaf_prototype_id,
            EdgeWeightKind::LeafPrototype,
        )
        .await?;
        Self::add_edge_to_func(ctx, leaf_prototype_id, func_id, EdgeWeightKind::new_use()).await?;

        let overlay_category_id = ctx
            .workspace_snapshot()?
            .get_or_create_static_category_node(CategoryNodeKind::Overlays)
            .await?;
        Self::add_overlay_category_edge(
            ctx,
            overlay_category_id,
            leaf_prototype_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        Ok(Self {
            id: leaf_prototype_id,
            inputs: attribute_paths,
            kind,
        })
    }

    pub async fn get_by_id(ctx: &DalContext, id: LeafPrototypeId) -> LeafPrototypeResult<Self> {
        let node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight(id)
            .await?
            .get_leaf_prototype_node_weight()?;

        let Some(ContentTypes::AttributePaths(AttributePathsContent::V1(AttributePathsContentV1(
            inputs,
        )))) = ctx
            .layer_db()
            .cas()
            .try_read_as(&node_weight.inputs().content_hash())
            .await?
        else {
            return Err(
                LeafPrototypeError::LeafPrototypeHasNoInputsAtContentAddress(node_weight.inputs()),
            );
        };

        let kind = node_weight.kind();

        Ok(Self { id, inputs, kind })
    }

    pub async fn func_id(
        ctx: &DalContext,
        leaf_prototype_id: LeafPrototypeId,
    ) -> LeafPrototypeResult<FuncId> {
        let snap = ctx.workspace_snapshot()?;
        Ok(snap
            .outgoing_targets_for_edge_weight_kind(
                leaf_prototype_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
            .pop()
            .ok_or(LeafPrototypeError::LeafPrototypeHasNoFunc(
                leaf_prototype_id,
            ))?
            .into())
    }

    pub async fn schemas(
        ctx: &DalContext,
        leaf_prototype_id: LeafPrototypeId,
    ) -> LeafPrototypeResult<Vec<SchemaId>> {
        let snap = ctx.workspace_snapshot()?;
        let mut result = vec![];

        for schema_id in snap
            .incoming_sources_for_edge_weight_kind(
                leaf_prototype_id,
                EdgeWeightKindDiscriminants::LeafPrototype,
            )
            .await?
        {
            let NodeWeight::Content(content_inner) = snap.get_node_weight(schema_id).await? else {
                continue;
            };

            if content_inner.content_address_discriminants() == ContentAddressDiscriminants::Schema
            {
                result.push(schema_id.into());
            }
        }

        Ok(result)
    }

    pub async fn for_schema(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> LeafPrototypeResult<Vec<LeafPrototype>> {
        let mut result = vec![];
        for leaf_prototype_id in ctx
            .workspace_snapshot()?
            .outgoing_targets_for_edge_weight_kind(
                schema_id,
                EdgeWeightKindDiscriminants::LeafPrototype,
            )
            .await?
        {
            let prototype = LeafPrototype::get_by_id(ctx, leaf_prototype_id.into()).await?;
            result.push(prototype);
        }

        Ok(result)
    }

    pub async fn attach_to_schema(
        &self,
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> LeafPrototypeResult<()> {
        Schema::add_edge_to_leaf_prototype(
            ctx,
            schema_id,
            self.id(),
            EdgeWeightKind::LeafPrototype,
        )
        .await?;

        Ok(())
    }

    pub async fn for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> LeafPrototypeResult<Vec<LeafPrototype>> {
        let snap = ctx.workspace_snapshot()?;
        let mut result = vec![];

        for incoming_source in snap
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let weight = snap.get_node_weight(incoming_source).await?;
            if let NodeWeight::LeafPrototype(_) = weight {
                let proto = LeafPrototype::get_by_id(ctx, incoming_source.into()).await?;
                result.push(proto);
            }
        }

        Ok(result)
    }

    pub async fn resolve_inputs(
        &self,
        ctx: &DalContext,
        root_attribute_value_id: AttributeValueId,
    ) -> LeafPrototypeResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        for path in self.inputs() {
            if let Some(av_id) = path
                .resolve(ctx, root_attribute_value_id)
                .await
                .map_err(Box::new)?
            {
                result.push(av_id);
            }
        }

        Ok(result)
    }

    /// Pairs each input with its corresponding LeafInputLocation
    pub async fn resolve_inputs_with_leaf_input_locations(
        &self,
        ctx: &DalContext,
        root_attribute_value_id: AttributeValueId,
    ) -> LeafPrototypeResult<Vec<(LeafInputLocation, AttributeValueId)>> {
        let mut result = vec![];

        for path in self.inputs() {
            let Some(av_id) = path
                .resolve(ctx, root_attribute_value_id)
                .await
                .map_err(Box::new)?
            else {
                continue;
            };

            let Some(leaf_input) = path.into() else {
                continue;
            };

            result.push((leaf_input, av_id));
        }

        Ok(result)
    }

    pub async fn resolve_output_map(
        &self,
        ctx: &DalContext,
        root_attribute_value_id: AttributeValueId,
    ) -> LeafPrototypeResult<AttributeValueId> {
        let path = self.kind().map_path();
        let Some(output_map_id) = path
            .resolve(ctx, root_attribute_value_id)
            .await
            .map_err(Box::new)?
        else {
            return Err(LeafPrototypeError::ComponentMissingDestinationAv(
                root_attribute_value_id,
                path.to_string(),
            ));
        };

        Ok(output_map_id)
    }

    pub async fn resolve_output_element(
        &self,
        ctx: &DalContext,
        root_attribute_value_id: AttributeValueId,
    ) -> LeafPrototypeResult<Option<AttributeValueId>> {
        let output_map_id = self
            .resolve_output_map(ctx, root_attribute_value_id)
            .await?;

        let func = Func::get_by_id(ctx, Self::func_id(ctx, self.id).await?).await?;
        Ok(
            AttributeValue::map_child_opt(ctx, output_map_id, &func.name)
                .await
                .map_err(Box::new)?,
        )
    }

    pub async fn execute(
        ctx: &DalContext,
        leaf_prototype_id: LeafPrototypeId,
        output_map_id: AttributeValueId,
        component_root_attribute_value_id: AttributeValueId,
        read_lock: Arc<RwLock<()>>,
    ) -> LeafPrototypeResult<PrototypeExecution> {
        let read_guard = read_lock.read().await;

        let prototype = Self::get_by_id(ctx, leaf_prototype_id).await?;
        let func_id = Self::func_id(ctx, leaf_prototype_id).await?;

        // We already did this in the DVU, consider stashing them somewhere
        let inputs = prototype
            .resolve_inputs_with_leaf_input_locations(ctx, component_root_attribute_value_id)
            .await?;

        let mut func_args: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        let mut input_attribute_value_ids = vec![];
        for (leaf_input, av_id) in &inputs {
            let arg_name = leaf_input.arg_name();
            let value = AttributeValue::view(ctx, *av_id)
                .await
                .map_err(Box::new)?
                .unwrap_or(serde_json::Value::Null);
            func_args.insert(arg_name.to_string(), value);
            input_attribute_value_ids.push(*av_id);
        }

        let args = serde_json::to_value(func_args)?;
        let result_channel =
            FuncRunner::run_attribute_value(ctx, output_map_id, func_id, args).await?;

        drop(read_guard);

        let mut func_run_value = result_channel
            .await
            .map_err(|_| Box::new(AttributeValueError::FuncRunnerSend))??;

        if func_run_value.unprocessed_value().is_some() {
            func_run_value.set_processed_value(Some(json!({})));
        } else {
            func_run_value.set_processed_value(None)
        };

        let (unprocessed_value_address, value_address) = write_values_to_cas(ctx, &func_run_value)
            .await
            .map_err(Box::new)?;

        FuncRunner::update_run(ctx, func_run_value.func_run_id(), |func_run| {
            func_run.set_success(unprocessed_value_address, value_address);
        })
        .await?;

        Ok(PrototypeExecution {
            func_run_value,
            func: Func::get_by_id(ctx, func_id).await?,
            input_attribute_value_ids,
            value_id: output_map_id,
        })
    }

    pub async fn remove(ctx: &DalContext, id: LeafPrototypeId) -> LeafPrototypeResult<()> {
        ctx.workspace_snapshot()?.remove_node_by_id(id).await?;
        Ok(())
    }

    pub async fn update_inputs(
        ctx: &DalContext,
        id: LeafPrototypeId,
        new_inputs: &[LeafInputLocation],
    ) -> LeafPrototypeResult<()> {
        let snap = ctx.workspace_snapshot()?;

        let attribute_paths: Vec<AttributePath> =
            new_inputs.iter().copied().map(Into::into).collect();

        let (content_hash, _) = ctx.layer_db().cas().write(
            Arc::new(crate::layer_db_types::ContentTypes::AttributePaths(
                attribute_paths.clone().into(),
            )),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let mut weight = snap.get_node_weight(id).await?;
        weight.new_content_hash(content_hash)?;

        snap.add_or_replace_node(weight).await?;

        Ok(())
    }

    implement_add_edge_to!(
        source_id: LeafPrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: LeafPrototypeResult,
    );

    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: LeafPrototypeId,
        add_fn: add_overlay_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: LeafPrototypeResult,
    );
}
