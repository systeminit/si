use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{pk, Timestamp};

pub use ui_menu::SchemaUiMenu;
pub use variant::{SchemaVariant, SchemaVariantId};

pub mod ui_menu;
pub mod variant;

// const FIND_SCHEMA_VARIANT_BY_NAME_FOR_SCHEMA: &str =
//     include_str!("./queries/find_schema_variant_for_schema_and_name.sql");

pub const SCHEMA_VERSION: SchemaContentDiscriminants = SchemaContentDiscriminants::V1;

pk!(SchemaId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Schema {
    id: SchemaId,
    #[serde(flatten)]
    timestamp: Timestamp,
    name: String,
    ui_hidden: bool,
    default_schema_variant_id: Option<SchemaVariantId>,
    component_kind: ComponentKind,
}

// FIXME(nick,zack,jacob): temporarily moved here.
#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ComponentKind {
    Credential,
    Standard,
}

#[derive(Debug, PartialEq)]
pub struct SchemaGraphNode {
    id: SchemaId,
    content_address: ContentAddress,
    content: SchemaContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum SchemaContent {
    V1(SchemaContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaContentV1 {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    pub name: String,
    pub ui_hidden: bool,
    pub default_schema_variant_id: Option<SchemaVariantId>,
    pub component_kind: ComponentKind,
}

impl SchemaGraphNode {
    pub fn assemble(
        id: impl Into<SchemaId>,
        content_hash: ContentHash,
        content: SchemaContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::Schema(content_hash),
            content,
        }
    }
}

impl Schema {
    pub fn assemble(id: SchemaId, inner: &SchemaContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name.clone(),
            ui_hidden: inner.ui_hidden,
            default_schema_variant_id: inner.default_schema_variant_id,
            component_kind: inner.component_kind,
        }
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
