use sea_orm::{
    entity::prelude::*,
    sea_query::{self, IdenStatic},
    TryGetError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "modules")]
pub struct Model {
    #[sea_orm(primary_key, column_type = r##"custom("ident")"##)]
    pub id: ModuleId,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub is_backup: bool,
    pub description: Option<String>,
    pub owner_user_id: String,
    pub owner_display_name: Option<String>,
    pub metadata: Json,
    pub latest_hash: String,
    pub latest_hash_created_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// custom ulid type

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleId(pub Ulid);

impl From<ModuleId> for Value {
    fn from(source: ModuleId) -> Self {
        Value::String(Some(Box::new(source.0.to_string())))
        // Value::String(serde_json::to_string(&source).ok().map(Box::new))
    }
}

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for ModuleId {
    type Error = sea_orm::DbErr;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(ModuleId(
            Ulid::from_string(&s).map_err(|err| DbErr::Type(err.to_string()))?,
        ))
    }
}
impl sea_orm::TryFromU64 for ModuleId {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal(
            format!("{} cannot be converted from u64", stringify!(ModuleId)),
        )))
    }
}

impl From<ModuleId> for String {
    fn from(val: ModuleId) -> Self {
        val.0.to_string()
    }
}

impl sea_orm::sea_query::Nullable for ModuleId {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}

impl sea_orm::TryGetable for ModuleId {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let json_str: String = res.try_get_by(idx).map_err(TryGetError::DbErr)?;
        Ulid::from_string(&json_str)
            .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
            .map(ModuleId)
        // serde_json::from_str(&json_str).map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string())))
    }
}

#[derive(IdenStatic, Clone, Copy)]
struct UlidIdentType;

impl sea_query::ValueType for ModuleId {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(x)) => Ok(ModuleId(
                Ulid::from_string(&x).map_err(|_| sea_query::ValueTypeErr)?,
                // serde_json::from_str(&x).map_err(|_| sea_query::ValueTypeErr)?,
            )),
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

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ModuleResponseError {
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl TryInto<module_index_client::ModuleDetailsResponse> for Model {
    type Error = ModuleResponseError;

    fn try_into(self) -> Result<module_index_client::ModuleDetailsResponse, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}
