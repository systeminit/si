use sea_orm::{entity::prelude::*, sea_query, TryGetError};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVariantId(pub Ulid);

impl From<SchemaVariantId> for Value {
    fn from(source: SchemaVariantId) -> Self {
        Value::String(Some(Box::new(source.0.to_string())))
    }
}

impl std::fmt::Display for SchemaVariantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for SchemaVariantId {
    type Error = sea_orm::DbErr;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(SchemaVariantId(
            Ulid::from_string(&s).map_err(|err| DbErr::Type(err.to_string()))?,
        ))
    }
}

impl sea_orm::TryFromU64 for SchemaVariantId {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal(
            format!(
                "{} cannot be converted from u64",
                stringify!(SchemaVariantId)
            ),
        )))
    }
}

impl From<SchemaVariantId> for String {
    fn from(val: SchemaVariantId) -> Self {
        val.0.to_string()
    }
}

impl sea_orm::sea_query::Nullable for SchemaVariantId {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}

impl sea_orm::TryGetable for SchemaVariantId {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let json_str: String =
            res.try_get_by(idx)
                .map_err(TryGetError::DbErr)
                .and_then(|opt: Option<String>| {
                    opt.ok_or(sea_orm::TryGetError::Null("null".to_string()))
                })?;
        Ulid::from_string(&json_str)
            .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
            .map(SchemaVariantId)
    }
}

impl sea_query::ValueType for SchemaVariantId {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(x)) => Ok(SchemaVariantId(
                Ulid::from_string(&x).map_err(|_| sea_query::ValueTypeErr)?,
            )),
            _ => Err(sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(SchemaVariantId).to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::String(StringLen::None)
    }
}
