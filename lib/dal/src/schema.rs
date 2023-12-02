use content_store::{ContentHash, Store, StoreError};
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use std::collections::HashMap;
use strum::EnumDiscriminants;
use thiserror::Error;
use tokio::sync::TryLockError;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{pk, ComponentKind, DalContext, Timestamp, TransactionsError};

pub use ui_menu::SchemaUiMenu;
pub use variant::{SchemaVariant, SchemaVariantId};

pub mod ui_menu;
pub mod variant;

pub const SCHEMA_VERSION: SchemaContentDiscriminants = SchemaContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
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
    // NOTE(nick): maybe we should have a special edge for this instead.
    default_schema_variant_id: Option<SchemaVariantId>,
    component_kind: ComponentKind,
    // NOTE(nick): what is the difference between these two?
    pub category_name: String,
    pub category: String,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SchemaContent {
    V1(SchemaContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaContentV1 {
    pub timestamp: Timestamp,
    pub name: String,
    pub ui_hidden: bool,
    // NOTE(nick): maybe we should have a special edge for this instead.
    pub default_schema_variant_id: Option<SchemaVariantId>,
    pub component_kind: ComponentKind,
    // NOTE(nick): what is the difference between these two?
    pub category_name: String,
    pub category: String,
}

impl From<Schema> for SchemaContentV1 {
    fn from(value: Schema) -> Self {
        Self {
            timestamp: value.timestamp,
            name: value.name,
            ui_hidden: value.ui_hidden,
            default_schema_variant_id: value.default_schema_variant_id,
            component_kind: value.component_kind,
            category_name: value.category_name,
            category: value.category,
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
            default_schema_variant_id: inner.default_schema_variant_id,
            component_kind: inner.component_kind,
            category_name: inner.category_name,
            category: inner.category,
        }
    }

    pub fn id(&self) -> SchemaId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn default_schema_variant_id(&self) -> Option<SchemaVariantId> {
        self.default_schema_variant_id
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        component_kind: ComponentKind,
        category_name: impl Into<String>,
        category: impl Into<String>,
    ) -> SchemaResult<Self> {
        let content = SchemaContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            ui_hidden: false,
            default_schema_variant_id: None,
            component_kind,
            category_name: category_name.into(),
            category: category.into(),
        };

        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&SchemaContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Schema(hash))?;

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let _node_index = workspace_snapshot.add_node(node_weight)?;

        let schema_category_index_id = workspace_snapshot.get_category(CategoryNodeKind::Schema)?;
        workspace_snapshot.add_edge(
            schema_category_index_id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            id,
        )?;

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn get_by_id(ctx: &DalContext, id: SchemaId) -> SchemaResult<Self> {
        dbg!("gettting by id");
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;
        dbg!("got node index and weight");

        let hash = node_weight.content_hash();

        let content: SchemaContent = ctx
            .content_store()
            .try_lock()?
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;
        dbg!("got content");

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
            let hash = ctx
                .content_store()
                .try_lock()?
                .add(&SchemaContent::V1(updated.clone()))?;

            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
            workspace_snapshot.update_content(ctx.change_set_pointer()?, schema.id.into(), hash)?;
        }

        Ok(schema)
    }

    pub async fn list(ctx: &DalContext) -> SchemaResult<Vec<Self>> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let mut schemas = vec![];
        let schema_category_index_id = workspace_snapshot.get_category(CategoryNodeKind::Schema)?;

        let schema_node_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            schema_category_index_id,
            EdgeWeightKindDiscriminants::Use,
        )?;

        let mut schema_node_weights = vec![];
        let mut schema_content_hashes = vec![];
        for index in schema_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Schema)?;
            schema_content_hashes.push(node_weight.content_hash());
            schema_node_weights.push(node_weight);
        }

        let schema_contents: HashMap<ContentHash, SchemaContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(schema_content_hashes.as_slice())
            .await?;

        for node_weight in schema_node_weights {
            match schema_contents.get(&node_weight.content_hash()) {
                Some(func_content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let SchemaContent::V1(inner) = func_content;

                    schemas.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(schemas)
    }

    pub async fn set_default_schema_variant_id(
        self,
        ctx: &DalContext,
        default_schema_variant_id: Option<SchemaVariantId>,
    ) -> SchemaResult<Self> {
        self.modify(ctx, |s| {
            s.default_schema_variant_id = default_schema_variant_id;
            Ok(())
        })
        .await
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
