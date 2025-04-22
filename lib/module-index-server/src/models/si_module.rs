use module_index_types::{LatestModuleResponse, ModuleDetailsResponse};
use sea_orm::{TryGetError, entity::prelude::*, sea_query};
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

impl From<ModuleKind> for sea_orm::Value {
    fn from(value: ModuleKind) -> Self {
        value.to_db_kind().into()
    }
}

impl sea_query::ValueType for ModuleKind {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(x)) => Ok(ModuleKind::from_str(x.as_str())?),
            _ => Err(sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(ModuleId).to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::String(StringLen::None)
    }
}

impl sea_orm::TryGetable for ModuleKind {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let json_str: String = res.try_get_by(idx).map_err(TryGetError::DbErr)?;

        ModuleKind::from_str(&json_str).map_err(|e| {
            TryGetError::DbErr(DbErr::TryIntoErr {
                from: "database module kind",
                into: "ModuleKind",
                source: Box::new(e),
            })
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
