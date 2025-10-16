use std::sync::Arc;

use si_id::{
    FuncId,
    LeafPrototypeId,
    SchemaId,
    ulid::Ulid,
};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use super::{
    Schema,
    SchemaError,
};
use crate::{
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    HelperError,
    WorkspaceSnapshotError,
    attribute::path::AttributePath,
    func::leaf::{
        LeafInputLocation,
        LeafKind,
    },
    implement_add_edge_to,
    layer_db_types::{
        AttributePathsContent,
        AttributePathsContentV1,
        ContentTypes,
    },
    workspace_snapshot::{
        content_address::ContentAddress,
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
    #[error("Helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("leaf prototype has no inputs at content address: {0:?}")]
    LeafPrototypeHasNoInputsAtContentAddress(ContentAddress),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
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
        inputs: Vec<LeafInputLocation>,
        func_id: FuncId,
    ) -> LeafPrototypeResult<Self> {
        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;

        let attribute_paths: Vec<AttributePath> = inputs.into_iter().map(Into::into).collect();

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
    ) -> LeafPrototypeResult<Option<FuncId>> {
        let snap = ctx.workspace_snapshot()?;
        Ok(snap
            .outgoing_targets_for_edge_weight_kind(
                leaf_prototype_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
            .pop()
            .map(Into::into))
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
