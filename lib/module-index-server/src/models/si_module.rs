use sea_orm::{entity::prelude::*, sea_query, TryGetError};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod module_id;
pub mod schema_id;

pub use module_id::ModuleId;
pub use schema_id::SchemaId;

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
        sea_query::ColumnType::String(None)
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
    pub created_at: DateTimeWithTimeZone,
    pub rejected_at: Option<DateTimeWithTimeZone>,
    pub rejected_by_display_name: Option<String>,
    pub kind: ModuleKind,
    pub is_builtin_at: Option<DateTimeWithTimeZone>,
    pub is_builtin_at_by_display_name: Option<String>,
    #[sea_orm(column_type = r##"custom("ident")"##, nullable)]
    pub schema_id: Option<SchemaId>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl TryInto<module_index_client::ModuleDetailsResponse> for Model {
    type Error = crate::routes::upsert_module_route::UpsertModuleError;

    fn try_into(self) -> Result<module_index_client::ModuleDetailsResponse, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}
