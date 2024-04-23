use petgraph::Outgoing;
use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;

use crate::change_set::ChangeSetError;
use crate::layer_db_types::{SchemaContent, SchemaContentDiscriminants, SchemaContentV1};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{implement_add_edge_to, pk, DalContext, HelperError, Timestamp, TransactionsError};

pub use variant::{SchemaVariant, SchemaVariantId};

pub mod variant;
pub mod view;

pub const SCHEMA_VERSION: SchemaContentDiscriminants = SchemaContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

pk!(SchemaId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Schema {
    id: SchemaId,
    #[serde(flatten)]
    timestamp: Timestamp,
    pub name: String,
    pub ui_hidden: bool,
    pub is_builtin: bool,
}

impl From<Schema> for SchemaContentV1 {
    fn from(value: Schema) -> Self {
        Self {
            timestamp: value.timestamp,
            name: value.name,
            ui_hidden: value.ui_hidden,
            is_builtin: value.is_builtin,
        }
    }
}

impl Schema {
    pub fn assemble(id: SchemaId, inner: SchemaContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            ui_hidden: inner.ui_hidden,
            is_builtin: inner.is_builtin,
        }
    }

    pub fn id(&self) -> SchemaId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_builtin(&self) -> bool {
        self.is_builtin
    }
    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }

    implement_add_edge_to!(
        source_id: SchemaId,
        destination_id: SchemaVariantId,
        add_fn: add_edge_to_variant,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: SchemaResult,
    );

    pub async fn new(ctx: &DalContext, name: impl Into<String>) -> SchemaResult<Self> {
        let content = SchemaContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            ui_hidden: false,
            is_builtin: false,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(SchemaContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Schema(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;

        let schema_category_index_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Schema)
            .await?;
        workspace_snapshot
            .add_edge(
                schema_category_index_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                id,
            )
            .await?;

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn get_default_schema_variant(
        &self,
        ctx: &DalContext,
    ) -> SchemaResult<Option<SchemaVariantId>> {
        Self::get_default_schema_variant_by_id(ctx, self.id).await
    }

    pub async fn get_default_schema_variant_by_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaResult<Option<SchemaVariantId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let default_schema_variant_node_indicies = workspace_snapshot
            .edges_directed(schema_id, Outgoing)
            .await?;

        for (edge_weight, _, target_index) in default_schema_variant_node_indicies {
            if *edge_weight.kind() == EdgeWeightKind::new_use_default() {
                return Ok(Some(
                    workspace_snapshot
                        .get_node_weight(target_index)
                        .await?
                        .id()
                        .into(),
                ));
            }
        }

        Ok(None)
    }

    pub async fn set_default_schema_variant(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        debug!(
            %schema_variant_id, "setting the default schema variant for schema: {}",
            self.id
        );

        // Our system will have edges as follows:
        //
        // Schema -> Use -> Schema Variant
        //
        // In order to make a schema variant the default for a schema, we need
        // to update the correct edge from Use to the default variant of Use,
        //
        // Schema -> Use {is_default = true} -> Schema Variant
        //
        // Therefore, when we are setting a default schema variant we need to
        // find any existing default Use edges and convert them back to uses AND we
        // need to find the existing Use edge between our nodes and change that
        // to be a default Use
        for (edge_weight, source_index, target_index) in workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                self.id,
                Outgoing,
                EdgeWeightKind::new_use_default().into(),
            )
            .await?
        {
            // We have found the existing Default edge between schema and schema variant
            // we now need to update that edge to be a Use
            workspace_snapshot
                .remove_edge(
                    ctx.change_set()?,
                    source_index,
                    target_index,
                    edge_weight.kind().into(),
                )
                .await?;

            Self::add_edge_to_variant(ctx, self.id, schema_variant_id, EdgeWeightKind::new_use())
                .await?;
        }

        for (edge_weight, source_index, target_index) in workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                self.id,
                Outgoing,
                EdgeWeightKind::new_use().into(),
            )
            .await?
        {
            // We have found the existing Use edge between schema and schema variant
            // we now need to update that edge to be a Default
            workspace_snapshot
                .remove_edge(
                    ctx.change_set()?,
                    source_index,
                    target_index,
                    edge_weight.kind().into(),
                )
                .await?;

            Self::add_edge_to_variant(
                ctx,
                self.id,
                schema_variant_id,
                EdgeWeightKind::new_use_default(),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn get_by_id(ctx: &DalContext, id: SchemaId) -> SchemaResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: SchemaContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let SchemaContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> SchemaResult<Self>
    where
        L: FnOnce(&mut Self) -> SchemaResult<()>,
    {
        let mut schema = self;

        let before = SchemaContentV1::from(schema.clone());
        lambda(&mut schema)?;
        let updated = SchemaContentV1::from(schema.clone());

        if updated != before {
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(SchemaContent::V1(updated.clone()).into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;

            ctx.workspace_snapshot()?
                .update_content(ctx.change_set()?, schema.id.into(), hash)
                .await?;
        }

        Ok(schema)
    }

    pub async fn list(ctx: &DalContext) -> SchemaResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut schemas = vec![];
        let schema_category_index_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Schema)
            .await?;

        let schema_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_category_index_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut schema_node_weights = vec![];
        let mut schema_content_hashes = vec![];
        for index in schema_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Schema)?;
            schema_content_hashes.push(node_weight.content_hash());
            schema_node_weights.push(node_weight);
        }

        let schema_contents: HashMap<ContentHash, SchemaContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(schema_content_hashes.as_slice())
            .await?;

        for node_weight in schema_node_weights {
            match schema_contents.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let SchemaContent::V1(inner) = content;

                    schemas.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(schemas)
    }

    /// Lists all [`Schemas`](Schema) by ID in the workspace.
    pub async fn list_ids(ctx: &DalContext) -> SchemaResult<Vec<SchemaId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let schema_category_index_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Schema)
            .await?;
        let schema_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_category_index_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut schema_ids = Vec::new();
        for index in schema_node_indices {
            let raw_id = workspace_snapshot.get_node_weight(index).await?.id();
            schema_ids.push(raw_id.into());
        }

        Ok(schema_ids)
    }

    // NOTE(nick): this assumes that schema names are unique.
    pub async fn find_by_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> SchemaResult<Option<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let schema_node_indices = {
            let schema_category_index_id = workspace_snapshot
                .get_category_node(None, CategoryNodeKind::Schema)
                .await?;
            workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    schema_category_index_id,
                    EdgeWeightKindDiscriminants::Use,
                )
                .await?
        };

        // NOTE(nick): this algorithm could be better.
        for schema_node_index in schema_node_indices {
            let schema_node_weight = {
                workspace_snapshot
                    .get_node_weight(schema_node_index)
                    .await?
                    .get_content_node_weight_of_kind(ContentAddressDiscriminants::Schema)?
            };
            let schema = Self::get_by_id(ctx, schema_node_weight.id().into()).await?;
            if schema.name == name.as_ref() {
                return Ok(Some(schema));
            }
        }
        Ok(None)
    }
}

// impl Schema {
//     pub async fn default_variant(&self, ctx: &DalContext) -> SchemaResult<SchemaVariant> {
//         match self.default_schema_variant_id() {
//             Some(schema_variant_id) => Ok(SchemaVariant::get_by_id(ctx, schema_variant_id)
//                 .await?
//                 .ok_or_else(|| SchemaError::NoDefaultVariant(*self.id()))?),
//             None => Err(SchemaError::NoDefaultVariant(*self.id())),
//         }
//     }
//
//     pub async fn is_builtin(&self, ctx: &DalContext) -> SchemaResult<bool> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 "SELECT id FROM schemas WHERE id = $1 and tenancy_workspace_pk = $2 LIMIT 1",
//                 &[self.id(), &WorkspacePk::NONE],
//             )
//             .await?;
//
//         Ok(row.is_some())
//     }
//
//     pub async fn find_by_name(ctx: &DalContext, name: impl AsRef<str>) -> SchemaResult<Schema> {
//         let name = name.as_ref();
//         let schemas = Schema::find_by_attr(ctx, "name", &name).await?;
//         schemas
//             .first()
//             .ok_or_else(|| SchemaError::NotFoundByName(name.into()))
//             .cloned()
//     }
//
//     pub async fn find_by_name_builtin(
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//     ) -> SchemaResult<Option<Schema>> {
//         let name = name.as_ref();
//
//         let builtin_ctx = ctx.clone_with_new_tenancy(Tenancy::new(WorkspacePk::NONE));
//         let builtin_schema = Self::find_by_name(&builtin_ctx, name).await?;
//
//         Ok(Self::get_by_id(ctx, builtin_schema.id()).await?)
//     }
//
//     pub async fn find_variant_by_name(
//         &self,
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//     ) -> SchemaResult<Option<SchemaVariant>> {
//         let name: &str = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_SCHEMA_VARIANT_BY_NAME_FOR_SCHEMA,
//                 &[ctx.tenancy(), ctx.visibility(), self.id(), &name],
//             )
//             .await?;
//
//         Ok(object_option_from_row_option(row)?)
//     }
//
//     pub async fn default_schema_variant_id_for_name(
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//     ) -> SchemaResult<SchemaVariantId> {
//         let name = name.as_ref();
//         let schemas = Schema::find_by_attr(ctx, "name", &name).await?;
//         let schema = schemas
//             .first()
//             .ok_or_else(|| SchemaError::NotFoundByName(name.into()))?;
//         let schema_variant_id = schema
//             .default_schema_variant_id()
//             .ok_or_else(|| SchemaError::NoDefaultVariant(*schema.id()))?;
//
//         Ok(*schema_variant_id)
//     }
// }
