use super::{InstalledPkgId, InstalledPkgResult};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::schema::variant::definition::SchemaVariantDefinitionId;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, DalContext, FuncId, SchemaId,
    SchemaVariantId, StandardModel, Tenancy, Timestamp, Visibility,
};
use strum::{AsRefStr, Display, EnumIter, EnumString};

const LIST_FOR_KIND_AND_HASH: &str =
    include_str!("../queries/installed_pkg/list_asset_for_kind_and_hash.sql");

const LIST_FOR_INSTALLED_PKG_ID: &str =
    include_str!("../queries/installed_pkg/list_asset_for_installed_pkg_id.sql");

pk!(InstalledPkgAssetPk);
pk!(InstalledPkgAssetId);
pk!(InstalledPkgAssetAssetId);

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum InstalledPkgAssetKind {
    Func,
    Schema,
    SchemaVariant,
    SchemaVariantDefinition,
}

/// An `InstalledPkgAsset` is a record of the installation of a package asset. It tracks the
/// asset installation and can be used to prevent duplicate installations and to remove packages
/// after installation.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InstalledPkgAsset {
    pk: InstalledPkgAssetPk,
    id: InstalledPkgAssetId,
    installed_pkg_id: InstalledPkgId,
    asset_id: InstalledPkgAssetAssetId,
    asset_hash: String,
    asset_kind: InstalledPkgAssetKind,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

// Could simplify all this with macros if we end up adding more kinds
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum InstalledPkgAssetTyped {
    Func {
        installed_pkg_asset_id: InstalledPkgAssetId,
        installed_pkg_id: InstalledPkgId,
        id: FuncId,
        hash: String,
    },
    Schema {
        installed_pkg_asset_id: InstalledPkgAssetId,
        installed_pkg_id: InstalledPkgId,
        id: SchemaId,
        hash: String,
    },
    SchemaVariant {
        installed_pkg_asset_id: InstalledPkgAssetId,
        installed_pkg_id: InstalledPkgId,
        id: SchemaVariantId,
        hash: String,
    },
    SchemaVariantDefinition {
        installed_pkg_asset_id: InstalledPkgAssetId,
        installed_pkg_id: InstalledPkgId,
        id: SchemaVariantDefinitionId,
        hash: String,
    },
}

impl InstalledPkgAssetTyped {
    pub fn new_for_schema(
        schema_id: SchemaId,
        installed_pkg_id: InstalledPkgId,
        hash: String,
    ) -> Self {
        Self::Schema {
            installed_pkg_asset_id: InstalledPkgAssetId::NONE,
            installed_pkg_id,
            id: schema_id,
            hash,
        }
    }

    pub fn new_for_schema_variant(
        schema_variant_id: SchemaVariantId,
        installed_pkg_id: InstalledPkgId,
        hash: String,
    ) -> Self {
        Self::SchemaVariant {
            installed_pkg_asset_id: InstalledPkgAssetId::NONE,
            installed_pkg_id,
            id: schema_variant_id,
            hash,
        }
    }

    pub fn new_for_schema_variant_definition(
        schema_variant_definition_id: SchemaVariantDefinitionId,
        installed_pkg_id: InstalledPkgId,
        hash: String,
    ) -> Self {
        Self::SchemaVariantDefinition {
            installed_pkg_asset_id: InstalledPkgAssetId::NONE,
            installed_pkg_id,
            id: schema_variant_definition_id,
            hash,
        }
    }

    pub fn new_for_func(func_id: FuncId, installed_pkg_id: InstalledPkgId, hash: String) -> Self {
        Self::Func {
            installed_pkg_asset_id: InstalledPkgAssetId::NONE,
            installed_pkg_id,
            id: func_id,
            hash,
        }
    }
}

impl From<&InstalledPkgAsset> for InstalledPkgAssetTyped {
    fn from(value: &InstalledPkgAsset) -> Self {
        let installed_pkg_asset_id = *value.id();
        let installed_pkg_id = value.installed_pkg_id();
        let hash = value.asset_hash().to_string();

        match value.asset_kind {
            InstalledPkgAssetKind::Schema => Self::Schema {
                installed_pkg_asset_id,
                installed_pkg_id,
                id: Into::<ulid::Ulid>::into(value.asset_id()).into(),
                hash,
            },
            InstalledPkgAssetKind::SchemaVariant => Self::SchemaVariant {
                installed_pkg_asset_id,
                installed_pkg_id,
                id: Into::<ulid::Ulid>::into(value.asset_id()).into(),
                hash,
            },
            InstalledPkgAssetKind::Func => Self::Func {
                installed_pkg_asset_id,
                installed_pkg_id,
                id: Into::<ulid::Ulid>::into(value.asset_id()).into(),
                hash,
            },
            InstalledPkgAssetKind::SchemaVariantDefinition => Self::SchemaVariantDefinition {
                installed_pkg_asset_id,
                installed_pkg_id,
                id: Into::<ulid::Ulid>::into(value.asset_id()).into(),
                hash,
            },
        }
    }
}

impl_standard_model! {
    model: InstalledPkgAsset,
    pk: InstalledPkgAssetPk,
    id: InstalledPkgAssetId,
    table_name: "installed_pkg_assets",
    history_event_label_base: "installed_pkg_asset",
    history_event_message_name: "Installed Pkg Asset"
}

impl InstalledPkgAsset {
    pub async fn new(
        ctx: &DalContext,
        pkg_asset: InstalledPkgAssetTyped,
    ) -> InstalledPkgResult<(Self, InstalledPkgAssetTyped)> {
        let (installed_pkg_id, asset_id, asset_hash, asset_kind): (
            InstalledPkgId,
            InstalledPkgAssetAssetId,
            String,
            InstalledPkgAssetKind,
        ) = match pkg_asset {
            InstalledPkgAssetTyped::Schema {
                installed_pkg_id,
                id,
                hash,
                ..
            } => (
                installed_pkg_id,
                Into::<ulid::Ulid>::into(id).into(),
                hash,
                InstalledPkgAssetKind::Schema,
            ),
            InstalledPkgAssetTyped::SchemaVariant {
                installed_pkg_id,
                id,
                hash,
                ..
            } => (
                installed_pkg_id,
                Into::<ulid::Ulid>::into(id).into(),
                hash,
                InstalledPkgAssetKind::SchemaVariant,
            ),
            InstalledPkgAssetTyped::SchemaVariantDefinition {
                installed_pkg_id,
                id,
                hash,
                ..
            } => (
                installed_pkg_id,
                Into::<ulid::Ulid>::into(id).into(),
                hash,
                InstalledPkgAssetKind::SchemaVariantDefinition,
            ),

            InstalledPkgAssetTyped::Func {
                installed_pkg_id,
                id,
                hash,
                ..
            } => (
                installed_pkg_id,
                Into::<ulid::Ulid>::into(id).into(),
                hash,
                InstalledPkgAssetKind::Func,
            ),
        };

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM installed_pkg_asset_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &installed_pkg_id,
                    &asset_id,
                    &asset_kind.as_ref(),
                    &asset_hash,
                ],
            )
            .await?;
        let object: InstalledPkgAsset = standard_model::finish_create_from_row(ctx, row).await?;
        let asset_typed: InstalledPkgAssetTyped = (&object).into();
        Ok((object, asset_typed))
    }

    pub fn as_installed_schema(&self) -> InstalledPkgResult<InstalledPkgAssetTyped> {
        let typed: InstalledPkgAssetTyped = self.into();

        match typed {
            InstalledPkgAssetTyped::Schema { .. } => Ok(typed),
            InstalledPkgAssetTyped::SchemaVariant {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::Schema,
                InstalledPkgAssetKind::SchemaVariant,
            )),
            InstalledPkgAssetTyped::SchemaVariantDefinition {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::Schema,
                InstalledPkgAssetKind::SchemaVariantDefinition,
            )),
            InstalledPkgAssetTyped::Func {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::Schema,
                InstalledPkgAssetKind::Func,
            )),
        }
    }

    pub fn as_installed_schema_variant_definition(
        &self,
    ) -> InstalledPkgResult<InstalledPkgAssetTyped> {
        let typed: InstalledPkgAssetTyped = self.into();

        match typed {
            InstalledPkgAssetTyped::SchemaVariantDefinition { .. } => Ok(typed),
            InstalledPkgAssetTyped::SchemaVariant {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::SchemaVariantDefinition,
                InstalledPkgAssetKind::SchemaVariant,
            )),
            InstalledPkgAssetTyped::Schema {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::SchemaVariantDefinition,
                InstalledPkgAssetKind::Schema,
            )),
            InstalledPkgAssetTyped::Func {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::SchemaVariantDefinition,
                InstalledPkgAssetKind::Func,
            )),
        }
    }

    pub fn as_installed_schema_variant(&self) -> InstalledPkgResult<InstalledPkgAssetTyped> {
        let typed: InstalledPkgAssetTyped = self.into();

        match typed {
            InstalledPkgAssetTyped::SchemaVariant { .. } => Ok(typed),
            InstalledPkgAssetTyped::Schema {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::SchemaVariant,
                InstalledPkgAssetKind::Schema,
            )),
            InstalledPkgAssetTyped::SchemaVariantDefinition {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::SchemaVariant,
                InstalledPkgAssetKind::SchemaVariantDefinition,
            )),
            InstalledPkgAssetTyped::Func {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::SchemaVariant,
                InstalledPkgAssetKind::Func,
            )),
        }
    }

    pub fn as_installed_func(&self) -> InstalledPkgResult<InstalledPkgAssetTyped> {
        let typed: InstalledPkgAssetTyped = self.into();

        match typed {
            InstalledPkgAssetTyped::Func { .. } => Ok(typed),
            InstalledPkgAssetTyped::Schema {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::Func,
                InstalledPkgAssetKind::Schema,
            )),
            InstalledPkgAssetTyped::SchemaVariantDefinition {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::Func,
                InstalledPkgAssetKind::SchemaVariantDefinition,
            )),
            InstalledPkgAssetTyped::SchemaVariant {
                installed_pkg_asset_id,
                ..
            } => Err(super::InstalledPkgError::InstalledPkgKindMismatch(
                installed_pkg_asset_id,
                InstalledPkgAssetKind::Func,
                InstalledPkgAssetKind::SchemaVariant,
            )),
        }
    }

    pub async fn list_for_installed_pkg_id(
        ctx: &DalContext,
        installed_pkg_id: InstalledPkgId,
    ) -> InstalledPkgResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_INSTALLED_PKG_ID,
                &[ctx.tenancy(), ctx.visibility(), &installed_pkg_id],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn list_for_kind_and_hash(
        ctx: &DalContext,
        kind: InstalledPkgAssetKind,
        hash: &str,
    ) -> InstalledPkgResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_KIND_AND_HASH,
                &[ctx.tenancy(), ctx.visibility(), &kind.as_ref(), &hash],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    standard_model_accessor!(asset_id, Pk(InstalledPkgAssetAssetId), InstalledPkgResult);
    standard_model_accessor!(installed_pkg_id, Pk(InstalledPkgId), InstalledPkgResult);
    standard_model_accessor!(asset_hash, String, InstalledPkgResult);
    standard_model_accessor!(asset_kind, Enum(InstalledPkgAssetKind), InstalledPkgResult);
}
