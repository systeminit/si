use module_index_types::{LatestModuleResponse, ModuleDetailsResponse};
use sea_orm::{entity::prelude::*, sea_query, DeriveValueType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub use si_id::ModuleIndexModuleId as ModuleId;
pub use si_id::SchemaId;
pub use si_id::SchemaVariantId;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "modules")]
pub struct Model {
    #[sea_orm(primary_key, column_type = r##"custom("ident")"##)]
    pub id: ModuleId,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub description: Option<String>,
    pub owner_user_id: String,
    pub owner_display_name: Option<String>,
    pub metadata: Json,
    pub latest_hash: String,
    pub latest_hash_created_at: DateTimeWithTimeZone,
    pub structural_hash: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub rejected_at: Option<DateTimeWithTimeZone>,
    pub rejected_by_display_name: Option<String>,
    pub kind: ModuleKind,
    pub is_builtin_at: Option<DateTimeWithTimeZone>,
    pub is_builtin_at_by_display_name: Option<String>,
    #[sea_orm(column_type = r##"custom("ident")"##, nullable)]
    pub schema_id: Option<SchemaId>,
    #[sea_orm(column_type = r##"custom("ident")"##, nullable)]
    pub schema_variant_id: Option<SchemaVariantId>,
    pub schema_variant_version: Option<String>,
    pub is_private_scoped: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveValueType)]
#[sea_orm(
    value_type = "String",
    from_str = "ModuleKind::from_str",
    to_str = "ModuleKind::to_db_kind"
)]
#[serde(rename_all = "camelCase")]
pub enum ModuleKind {
    Module,
    WorkspaceBackup,
}

impl ModuleKind {
    pub fn to_db_kind(&self) -> String {
        match self {
            ModuleKind::Module => "module".into(),
            ModuleKind::WorkspaceBackup => "workspaceBackup".into(),
        }
    }
}

impl FromStr for ModuleKind {
    type Err = sea_query::ValueTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "module" => ModuleKind::Module,
            "workspaceBackup" => ModuleKind::WorkspaceBackup,
            _ => return Err(sea_query::ValueTypeErr),
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::SchemaId",
        to = "Column::SchemaId"
        on_condition = r#"Column::SchemaId.is_not_null()"#
    )]
    SchemaIdReference,
}

pub struct SchemaIdReferenceLink;

impl Linked for SchemaIdReferenceLink {
    type FromEntity = Entity;
    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::SchemaIdReference.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub fn make_module_details_response(
    module: Model,
    linked_modules: Vec<Model>,
) -> ModuleDetailsResponse {
    ModuleDetailsResponse {
        id: module.id.to_string(),
        name: module.name,
        description: module.description,
        owner_user_id: module.owner_user_id.to_string(),
        owner_display_name: module.owner_display_name,
        metadata: module.metadata,
        latest_hash: module.latest_hash,
        latest_hash_created_at: module.latest_hash_created_at.into(),
        created_at: module.created_at.into(),
        schema_id: module.schema_id.map(|schema_id| schema_id.to_string()),
        schema_variant_id: module
            .schema_variant_id
            .map(|schema_variant_id| schema_variant_id.to_string()),
        schema_variant_version: module.schema_variant_version,
        past_hashes: Some(
            linked_modules
                .into_iter()
                .map(|module| module.latest_hash)
                .collect(),
        ),
        structural_hash: module.structural_hash,
    }
}

pub fn make_latest_modules_response(module: Model) -> LatestModuleResponse {
    LatestModuleResponse {
        id: module.id.to_string(),
        name: module.name,
        description: module.description,
        owner_user_id: module.owner_user_id.to_string(),
        owner_display_name: module.owner_display_name,
        metadata: module.metadata,
        latest_hash: module.latest_hash,
        latest_hash_created_at: module.latest_hash_created_at.into(),
        created_at: module.created_at.into(),
        schema_id: module.schema_id.map(|schema_id| schema_id.to_string()),
    }
}
