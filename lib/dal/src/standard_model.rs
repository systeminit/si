use crate::WriteTenancy;
use chrono::{DateTime, Utc};
use postgres_types::ToSql;
use serde::de::DeserializeOwned;
use serde::Serialize;
use si_data::{NatsError, PgError};
use strum_macros::AsRefStr;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{DalContext, HistoryEvent, HistoryEventError, Timestamp, Visibility};

#[derive(Error, Debug)]
pub enum StandardModelError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats error")]
    Nats(#[from] NatsError),
    #[error("{0} id {1} is missing when one was expected; it does not exist, is not visible, or is not valid for this tenancy")]
    ModelMissing(String, String),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
}

pub type StandardModelResult<T> = Result<T, StandardModelError>;

#[derive(AsRefStr, Debug, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum TypeHint {
    Bytea,
    BigInt,
    Boolean,
    Char,
    Integer,
    SmallInt,
    Text,
    JsonB,
}

#[instrument(skip(ctx))]
pub async fn get_by_pk<PK: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    pk: &PK,
) -> StandardModelResult<OBJECT> {
    let row = ctx
        .txns()
        .pg()
        .query_one("SELECT object FROM get_by_pk_v1($1, $2)", &[&table, &pk])
        .await?;
    let json: serde_json::Value = row.try_get("object")?;
    let object: OBJECT = serde_json::from_value(json)?;
    Ok(object)
}

#[instrument(skip(ctx))]
pub async fn get_by_id<ID: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    id: &ID,
) -> StandardModelResult<Option<OBJECT>> {
    let row_option = ctx
        .txns()
        .pg()
        .query_opt(
            "SELECT * FROM get_by_id_v1($1, $2, $3, $4)",
            &[&table, ctx.read_tenancy(), ctx.visibility(), &id],
        )
        .await?;
    object_option_from_row_option(row_option)
}

// This likely has some fun bugs living inside it when the value you pass is not
// a string. Bright side - so far, only strings! :)
// Hugs, Adam
#[instrument(skip(ctx))]
pub async fn find_by_attr<V: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    attr_name: &str,
    value: &V,
) -> StandardModelResult<Vec<OBJECT>> {
    let txns = ctx.txns();
    let rows = txns
        .pg()
        .query(
            "SELECT * FROM find_by_attr_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.read_tenancy(),
                ctx.visibility(),
                &attr_name,
                &value,
            ],
        )
        .await?;
    objects_from_rows(rows)
}

pub fn object_option_from_row_option<OBJECT: DeserializeOwned>(
    row_option: Option<tokio_postgres::Row>,
) -> StandardModelResult<Option<OBJECT>> {
    match row_option {
        Some(row) => {
            let json: serde_json::Value = row.try_get("object")?;
            let object: OBJECT = serde_json::from_value(json)?;
            Ok(Some(object))
        }
        None => Ok(None),
    }
}

#[instrument(skip(ctx))]
pub async fn belongs_to<ID: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    retrieve_table: &str,
    id: &ID,
) -> StandardModelResult<Option<OBJECT>> {
    let row_option = ctx
        .txns()
        .pg()
        .query_opt(
            "SELECT * FROM belongs_to_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.read_tenancy(),
                ctx.visibility(),
                &retrieve_table,
                &id,
            ],
        )
        .await?;
    object_option_from_row_option(row_option)
}

#[instrument(skip(ctx))]
pub async fn set_belongs_to<ObjectId: Send + Sync + ToSql, BelongsToId: Send + Sync + ToSql>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    object_id: &ObjectId,
    belongs_to_id: &BelongsToId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT set_belongs_to_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.write_tenancy(),
                ctx.visibility(),
                &object_id,
                &belongs_to_id,
            ],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn unset_belongs_to<ObjectId: Send + Sync + ToSql>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    object_id: &ObjectId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT unset_belongs_to_v1($1, $2, $3, $4)",
            &[&table, ctx.write_tenancy(), ctx.visibility(), &object_id],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn has_many<ID: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    retrieve_table: &str,
    belongs_to_id: &ID,
) -> StandardModelResult<Vec<OBJECT>> {
    let rows = ctx
        .txns()
        .pg()
        .query(
            "SELECT * FROM has_many_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.read_tenancy(),
                ctx.visibility(),
                &retrieve_table,
                &belongs_to_id,
            ],
        )
        .await?;
    objects_from_rows(rows)
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(ctx))]
pub async fn many_to_many<
    LeftId: Send + Sync + ToSql,
    RightId: Send + Sync + ToSql,
    Object: DeserializeOwned,
>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    left_table: &str,
    right_table: &str,
    left_object_id: Option<&LeftId>,
    right_object_id: Option<&RightId>,
) -> StandardModelResult<Vec<Object>> {
    let rows = ctx
        .txns()
        .pg()
        .query(
            "SELECT * FROM many_to_many_v1($1, $2, $3, $4, $5, $6, $7)",
            &[
                &table,
                ctx.read_tenancy(),
                ctx.visibility(),
                &left_table,
                &right_table,
                &left_object_id,
                &right_object_id,
            ],
        )
        .await?;
    objects_from_rows(rows)
}

#[instrument(skip(ctx))]
pub async fn associate_many_to_many<LeftId: Send + Sync + ToSql, RightId: Send + Sync + ToSql>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    left_object_id: &LeftId,
    right_object_id: &RightId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT associate_many_to_many_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.write_tenancy(),
                ctx.visibility(),
                &left_object_id,
                &right_object_id,
            ],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn disassociate_many_to_many<
    LeftId: Send + Sync + ToSql,
    RightId: Send + Sync + ToSql,
>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    left_object_id: &LeftId,
    right_object_id: &RightId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT disassociate_many_to_many_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.write_tenancy(),
                ctx.visibility(),
                &left_object_id,
                &right_object_id,
            ],
        )
        .await?;
    Ok(())
}

pub fn objects_from_rows<OBJECT: DeserializeOwned>(
    rows: Vec<tokio_postgres::Row>,
) -> StandardModelResult<Vec<OBJECT>> {
    let mut result = Vec::new();
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("object")?;
        let object: OBJECT = serde_json::from_value(json)?;
        result.push(object);
    }
    Ok(result)
}

pub fn object_from_row<OBJECT: DeserializeOwned>(
    row: tokio_postgres::Row,
) -> StandardModelResult<OBJECT> {
    let json: serde_json::Value = row.try_get("object")?;
    let object: OBJECT = serde_json::from_value(json)?;
    Ok(object)
}

pub fn option_object_from_row<OBJECT: DeserializeOwned>(
    maybe_row: Option<tokio_postgres::Row>,
) -> StandardModelResult<Option<OBJECT>> {
    let result = match maybe_row {
        Some(row) => Some(object_from_row(row)?),
        None => None,
    };
    Ok(result)
}

#[instrument(skip_all)]
#[allow(clippy::too_many_arguments)]
pub async fn update<ID, VALUE>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    column: &str,
    id: &ID,
    value: VALUE,
    hint: TypeHint,
) -> StandardModelResult<DateTime<Utc>>
where
    ID: Send + Sync + ToSql + std::fmt::Display,
    VALUE: Send + Sync + ToSql,
{
    let query = format!(
        "SELECT updated_at FROM update_by_id_v1($1, $2, $3, $4, $5, $6::{})",
        hint.as_ref()
    );

    let row = ctx
        .txns()
        .pg()
        .query_one(
            &query,
            &[
                &table,
                &column,
                ctx.write_tenancy(),
                ctx.visibility(),
                &id,
                &value,
            ],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), id.to_string()))
}

#[instrument(skip_all)]
pub async fn list<OBJECT: DeserializeOwned>(
    ctx: &DalContext<'_, '_>,
    table: &str,
) -> StandardModelResult<Vec<OBJECT>> {
    let rows = ctx
        .txns()
        .pg()
        .query(
            "SELECT * FROM list_models_v1($1, $2, $3)",
            &[&table, ctx.read_tenancy(), ctx.visibility()],
        )
        .await?;
    objects_from_rows(rows)
}

#[instrument(skip_all)]
pub async fn delete<PK: Send + Sync + ToSql + std::fmt::Display>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    pk: PK,
) -> StandardModelResult<DateTime<Utc>> {
    let row = ctx
        .txns()
        .pg()
        .query_one(
            "SELECT updated_at FROM delete_by_pk_v1($1, $2, $3)",
            &[&table, ctx.read_tenancy(), &pk],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), pk.to_string()))
}

#[instrument(skip_all)]
pub async fn undelete<PK: Send + Sync + ToSql + std::fmt::Display>(
    ctx: &DalContext<'_, '_>,
    table: &str,
    pk: PK,
) -> StandardModelResult<DateTime<Utc>> {
    let row = ctx
        .txns()
        .pg()
        .query_one(
            "SELECT updated_at FROM undelete_by_pk_v1($1, $2, $3)",
            &[&table, ctx.read_tenancy(), &pk],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), pk.to_string()))
}

#[instrument(skip_all)]
pub async fn finish_create_from_row<Object: Send + Sync + DeserializeOwned + StandardModel>(
    ctx: &DalContext<'_, '_>,
    row: tokio_postgres::Row,
) -> StandardModelResult<Object> {
    let json: serde_json::Value = row.try_get("object")?;
    let _history_event = HistoryEvent::new(
        ctx,
        Object::history_event_label(vec!["create"]),
        Object::history_event_message("created"),
        &serde_json::json![{ "visibility": ctx.visibility() }],
    )
    .await?;
    let object: Object = serde_json::from_value(json)?;
    Ok(object)
}

#[async_trait::async_trait]
pub trait StandardModel {
    type Pk: Send + Sync + ToSql + std::fmt::Display + Serialize + DeserializeOwned;
    type Id: Send + Sync + ToSql + std::fmt::Display + Serialize + DeserializeOwned;

    fn pk(&self) -> &Self::Pk;
    fn id(&self) -> &Self::Id;

    fn table_name() -> &'static str;
    fn history_event_label_base() -> &'static str;
    fn history_event_message_name() -> &'static str;

    fn visibility(&self) -> &Visibility;
    fn visibility_mut(&mut self) -> &mut Visibility;

    fn tenancy(&self) -> &WriteTenancy;
    fn tenancy_mut(&mut self) -> &mut WriteTenancy;

    fn timestamp(&self) -> &Timestamp;
    fn timestamp_mut(&mut self) -> &mut Timestamp;

    fn history_event_label(parts: Vec<&str>) -> String {
        format!("{}.{}", Self::history_event_label_base(), parts.join("."))
    }
    fn history_event_message(msg: impl AsRef<str>) -> String {
        let msg = msg.as_ref();
        format!("{} {}", Self::history_event_message_name(), msg)
    }

    #[instrument(skip_all)]
    async fn get_by_pk(ctx: &DalContext<'_, '_>, pk: &Self::Pk) -> StandardModelResult<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let object = crate::standard_model::get_by_pk(ctx, Self::table_name(), &pk).await?;
        Ok(object)
    }

    #[instrument(skip_all)]
    async fn get_by_id(ctx: &DalContext<'_, '_>, id: &Self::Id) -> StandardModelResult<Option<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let object = crate::standard_model::get_by_id(ctx, Self::table_name(), &id).await?;
        Ok(object)
    }

    #[instrument(skip_all)]
    async fn find_by_attr<V: Send + Sync + ToSql>(
        ctx: &DalContext<'_, '_>,
        attr_name: &str,
        value: &V,
    ) -> StandardModelResult<Vec<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let objects =
            crate::standard_model::find_by_attr(ctx, Self::table_name(), attr_name, value).await?;
        Ok(objects)
    }

    #[instrument(skip_all)]
    async fn list(ctx: &DalContext<'_, '_>) -> StandardModelResult<Vec<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let result = crate::standard_model::list(ctx, Self::table_name()).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn delete(&mut self, ctx: &DalContext<'_, '_>) -> StandardModelResult<()>
    where
        Self: Send + Sync,
    {
        let updated_at: chrono::DateTime<chrono::Utc> =
            crate::standard_model::delete(ctx, Self::table_name(), self.pk()).await?;
        // TODO(fnichol): I think that mutating our own visibility is likely okay in this
        // situation, as opposed to passing in an explicit visibility. The consequence is that
        // you'll be setting *this* object to be in a deleted state, no matter its current
        // visibility. This may prove to be sufficiently unsafe and warrants an explicitly passed
        // visibility when deleting. As it stands right now, it would be maximally safe to fetch
        // this object by id for the target visibility (with `deleted = false`) and then delete
        // *that* instance.
        self.visibility_mut().deleted = true;
        self.timestamp_mut().updated_at = updated_at;
        let _history_event = crate::HistoryEvent::new(
            ctx,
            &Self::history_event_label(vec!["deleted"]),
            &Self::history_event_message("deleted"),
            &serde_json::json![{
                "pk": self.pk(),
                "id": self.id(),
                "visibility": self.visibility(),
            }],
        )
        .await?;
        Ok(())
    }

    #[instrument(skip_all)]
    async fn undelete(&mut self, ctx: &DalContext<'_, '_>) -> StandardModelResult<()>
    where
        Self: Send + Sync,
    {
        let updated_at: chrono::DateTime<chrono::Utc> =
            crate::standard_model::undelete(ctx, Self::table_name(), self.pk()).await?;
        // TODO(fnichol): See the `Self.delete()` method for notes and caution.
        self.visibility_mut().deleted = false;
        self.timestamp_mut().updated_at = updated_at;
        let _history_event = crate::HistoryEvent::new(
            ctx,
            &Self::history_event_label(vec!["undeleted"]),
            &Self::history_event_message("undeleted"),
            &serde_json::json![{
                "pk": self.pk(),
                "id": self.id(),
                "visibility": self.visibility(),
            }],
        )
        .await?;
        Ok(())
    }
}

#[macro_export]
macro_rules! impl_standard_model {
    (model: $model:ident,
     pk: $pk:ident,
     id: $id:ident,
     table_name: $table_name:expr,
     history_event_label_base: $history_event_label_base:expr,
     history_event_message_name: $history_event_message_name:expr $(,)?) => {
        impl StandardModel for $model {
            type Pk = $pk;
            type Id = $id;

            fn pk(&self) -> &Self::Pk {
                &self.pk
            }

            fn id(&self) -> &Self::Id {
                &self.id
            }

            fn table_name() -> &'static str {
                $table_name
            }

            fn history_event_label_base() -> &'static str {
                $history_event_label_base
            }

            fn history_event_message_name() -> &'static str {
                $history_event_message_name
            }

            fn visibility(&self) -> &Visibility {
                &self.visibility
            }

            fn visibility_mut(&mut self) -> &mut Visibility {
                &mut self.visibility
            }

            fn tenancy(&self) -> &WriteTenancy {
                &self.tenancy
            }

            fn tenancy_mut(&mut self) -> &mut WriteTenancy {
                &mut self.tenancy
            }

            fn timestamp(&self) -> &Timestamp {
                &self.timestamp
            }

            fn timestamp_mut(&mut self) -> &mut Timestamp {
                &mut self.timestamp
            }
        }
    };
}
