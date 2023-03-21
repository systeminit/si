use crate::{Tenancy, UserError, UserPk};
use chrono::{DateTime, Utc};
use postgres_types::ToSql;
use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgRow};
use std::fmt::Debug;
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
    #[error(transparent)]
    User(#[from] UserError),
    #[error("user not found: {0}")]
    UserNotFound(UserPk),
}

pub type StandardModelResult<T> = Result<T, StandardModelError>;

#[derive(AsRefStr, Debug, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum TypeHint {
    BigInt,
    Boolean,
    BpChar,
    Bytea,
    Char,
    Integer,
    Json,
    JsonB,
    SmallInt,
    Text,
    Ident,
    #[strum(serialize = "timestamp with time zone")]
    TimestampWithTimeZone,
}

#[instrument(skip(ctx))]
pub async fn get_by_pk<PK: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    ctx: &DalContext,
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
    ctx: &DalContext,
    table: &str,
    id: &ID,
) -> StandardModelResult<Option<OBJECT>> {
    let row_option = ctx
        .txns()
        .pg()
        .query_opt(
            "SELECT * FROM get_by_id_v1($1, $2, $3, $4)",
            &[&table, ctx.tenancy(), ctx.visibility(), &id],
        )
        .await?;
    object_option_from_row_option(row_option)
}

// This likely has some fun bugs living inside it when the value you pass is not
// a string. Bright side - so far, only strings! :)
// Hugs, Adam
#[instrument(skip(ctx))]
pub async fn find_by_attr<V: Send + Sync + ToString + Debug, OBJECT: DeserializeOwned>(
    ctx: &DalContext,
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
                ctx.tenancy(),
                ctx.visibility(),
                &attr_name,
                &value.to_string(),
            ],
        )
        .await?;
    objects_from_rows(rows)
}

#[instrument(skip(ctx))]
pub async fn find_by_attr_null<OBJECT: DeserializeOwned>(
    ctx: &DalContext,
    table: &str,
    attr_name: &str,
) -> StandardModelResult<Vec<OBJECT>> {
    let txns = ctx.txns();
    let rows = txns
        .pg()
        .query(
            "SELECT * FROM find_by_attr_null_v1($1, $2, $3, $4)",
            &[&table, ctx.tenancy(), ctx.visibility(), &attr_name],
        )
        .await?;
    objects_from_rows(rows)
}

#[instrument(skip(ctx))]
pub async fn find_by_attr_in<V: Send + Sync + ToString + Debug, OBJECT: DeserializeOwned>(
    ctx: &DalContext,
    table: &str,
    attr_name: &str,
    value: &[&V],
) -> StandardModelResult<Vec<OBJECT>> {
    let txns = ctx.txns();
    let rows = txns
        .pg()
        .query(
            "SELECT * FROM find_by_attr_in_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.tenancy(),
                ctx.visibility(),
                &attr_name,
                &value.iter().map(|i| i.to_string()).collect::<Vec<String>>(),
            ],
        )
        .await?;
    objects_from_rows(rows)
}

#[instrument(skip(ctx))]
pub async fn find_by_attr_not_in<V: Send + Sync + ToString + Debug, OBJECT: DeserializeOwned>(
    ctx: &DalContext,
    table: &str,
    attr_name: &str,
    value: &[&V],
) -> StandardModelResult<Vec<OBJECT>> {
    let txns = ctx.txns();
    let rows = txns
        .pg()
        .query(
            "SELECT * FROM find_by_attr_not_in_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                ctx.tenancy(),
                ctx.visibility(),
                &attr_name,
                &value.iter().map(|i| i.to_string()).collect::<Vec<String>>(),
            ],
        )
        .await?;
    objects_from_rows(rows)
}

pub fn object_option_from_row_option<OBJECT: DeserializeOwned>(
    row_option: Option<PgRow>,
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
    ctx: &DalContext,
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
                ctx.tenancy(),
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
    ctx: &DalContext,
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
                ctx.tenancy(),
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
    ctx: &DalContext,
    table: &str,
    object_id: &ObjectId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT unset_belongs_to_v1($1, $2, $3, $4)",
            &[&table, ctx.tenancy(), ctx.visibility(), &object_id],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn hard_unset_belongs_to_in_change_set<ObjectId: Send + Sync + ToSql>(
    ctx: &DalContext,
    table: &str,
    object_id: &ObjectId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT hard_unset_belongs_to_in_change_set_v1($1, $2, $3, $4)",
            &[&table, ctx.tenancy(), ctx.visibility(), &object_id],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn unset_all_belongs_to<BelongsToId: Send + Sync + ToSql>(
    ctx: &DalContext,
    table: &str,
    belongs_to_id: &BelongsToId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT unset_all_belongs_to_v1($1, $2, $3, $4)",
            &[&table, ctx.tenancy(), ctx.visibility(), &belongs_to_id],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn hard_unset_all_belongs_to_in_change_set<BelongsToId: Send + Sync + ToSql>(
    ctx: &DalContext,
    table: &str,
    belongs_to_id: &BelongsToId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT hard_unset_all_belongs_to_in_change_set_v1($1, $2, $3, $4)",
            &[&table, ctx.tenancy(), ctx.visibility(), &belongs_to_id],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn has_many<ID: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    ctx: &DalContext,
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
                ctx.tenancy(),
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
    ctx: &DalContext,
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
                ctx.tenancy(),
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
    ctx: &DalContext,
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
                ctx.tenancy(),
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
    ctx: &DalContext,
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
                ctx.tenancy(),
                ctx.visibility(),
                &left_object_id,
                &right_object_id,
            ],
        )
        .await?;
    Ok(())
}

#[instrument(skip(ctx))]
pub async fn disassociate_all_many_to_many<LeftId: Send + Sync + ToSql>(
    ctx: &DalContext,
    table: &str,
    left_object_id: &LeftId,
) -> StandardModelResult<()> {
    ctx.txns()
        .pg()
        .query_one(
            "SELECT disassociate_all_many_to_many_v1($1, $2, $3, $4)",
            &[&table, ctx.tenancy(), ctx.visibility(), &left_object_id],
        )
        .await?;
    Ok(())
}

pub fn objects_from_rows<OBJECT: DeserializeOwned>(
    rows: Vec<PgRow>,
) -> StandardModelResult<Vec<OBJECT>> {
    let mut result = Vec::new();
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("object")?;
        let object: OBJECT = serde_json::from_value(json)?;
        result.push(object);
    }
    Ok(result)
}

pub fn object_from_row<OBJECT: DeserializeOwned>(row: PgRow) -> StandardModelResult<OBJECT> {
    let json: serde_json::Value = row.try_get("object")?;
    let object: OBJECT = serde_json::from_value(json)?;
    Ok(object)
}

pub fn option_object_from_row<OBJECT: DeserializeOwned>(
    maybe_row: Option<PgRow>,
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
    ctx: &DalContext,
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
                ctx.tenancy(),
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
    ctx: &DalContext,
    table: &str,
) -> StandardModelResult<Vec<OBJECT>> {
    let rows = ctx
        .txns()
        .pg()
        .query(
            "SELECT * FROM list_models_v1($1, $2, $3)",
            &[&table, ctx.tenancy(), ctx.visibility()],
        )
        .await?;
    objects_from_rows(rows)
}

#[instrument(skip_all)]
pub async fn delete_by_id<ID: Send + Sync + ToSql + std::fmt::Display>(
    ctx: &DalContext,
    table: &str,
    id: ID,
) -> StandardModelResult<DateTime<Utc>> {
    let row = ctx
        .txns()
        .pg()
        .query_one(
            "SELECT delete_by_id_v1($1, $2, $3, $4) AS deleted_at",
            &[&table, ctx.tenancy(), ctx.visibility(), &id],
        )
        .await?;
    row.try_get("deleted_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), id.to_string()))
}

#[instrument(skip_all)]
pub async fn delete_by_pk<PK: Send + Sync + ToSql + std::fmt::Display>(
    ctx: &DalContext,
    table: &str,
    pk: PK,
) -> StandardModelResult<DateTime<Utc>> {
    let row = ctx
        .txns()
        .pg()
        .query_one(
            "SELECT updated_at FROM delete_by_pk_v1($1, $2, $3)",
            &[&table, ctx.tenancy(), &pk],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), pk.to_string()))
}

#[instrument(skip_all)]
pub async fn undelete<PK: Send + Sync + ToSql + std::fmt::Display>(
    ctx: &DalContext,
    table: &str,
    pk: PK,
) -> StandardModelResult<DateTime<Utc>> {
    let row = ctx
        .txns()
        .pg()
        .query_one(
            "SELECT updated_at FROM undelete_by_pk_v1($1, $2, $3)",
            &[&table, ctx.tenancy(), &pk],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), pk.to_string()))
}

#[instrument(skip_all)]
pub async fn hard_delete<PK: Send + Sync + ToSql + std::fmt::Display, OBJECT: DeserializeOwned>(
    ctx: &DalContext,
    table: &str,
    pk: &PK,
) -> StandardModelResult<OBJECT> {
    let row = ctx
        .txns()
        .pg()
        .query_one(
            "SELECT object FROM hard_delete_by_pk_v1($1, $2)",
            &[&table, &pk],
        )
        .await?;
    let json: serde_json::Value = row.try_get("object")?;
    Ok(serde_json::from_value(json)?)
}

#[instrument(skip_all)]
pub async fn finish_create_from_row<Object: Send + Sync + DeserializeOwned + StandardModel>(
    ctx: &DalContext,
    row: PgRow,
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

    fn tenancy(&self) -> &Tenancy;
    fn tenancy_mut(&mut self) -> &mut Tenancy;

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
    async fn get_by_pk(ctx: &DalContext, pk: &Self::Pk) -> StandardModelResult<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let object = crate::standard_model::get_by_pk(ctx, Self::table_name(), &pk).await?;
        Ok(object)
    }

    #[instrument(skip_all)]
    async fn get_by_id(ctx: &DalContext, id: &Self::Id) -> StandardModelResult<Option<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let object = crate::standard_model::get_by_id(ctx, Self::table_name(), &id).await?;
        Ok(object)
    }

    #[instrument(skip_all)]
    async fn find_by_attr<V: Send + Sync + ToString + Debug>(
        ctx: &DalContext,
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
    async fn find_by_attr_null(ctx: &DalContext, attr_name: &str) -> StandardModelResult<Vec<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let objects =
            crate::standard_model::find_by_attr_null(ctx, Self::table_name(), attr_name).await?;
        Ok(objects)
    }

    /// Finds rows in the standard model if ANY of them match one of the values
    /// provided in `value` (equivalent to `WHERE attr_name IN (a, b, c)`). Same
    /// caveats as `find_by_attr`: `V` is almost always &String, untested with
    /// other types.
    #[instrument(skip_all)]
    async fn find_by_attr_in<V: Send + Sync + ToString + Debug>(
        ctx: &DalContext,
        attr_name: &str,
        value: &[&V],
    ) -> StandardModelResult<Vec<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        Ok(
            crate::standard_model::find_by_attr_in(ctx, Self::table_name(), attr_name, value)
                .await?,
        )
    }

    async fn find_by_attr_not_in<V: Send + Sync + ToString + Debug>(
        ctx: &DalContext,
        attr_name: &str,
        value: &[&V],
    ) -> StandardModelResult<Vec<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        Ok(
            crate::standard_model::find_by_attr_not_in(ctx, Self::table_name(), attr_name, value)
                .await?,
        )
    }

    #[instrument(skip_all)]
    async fn list(ctx: &DalContext) -> StandardModelResult<Vec<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let result = crate::standard_model::list(ctx, Self::table_name()).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn delete_by_pk(&mut self, ctx: &DalContext) -> StandardModelResult<()>
    where
        Self: Send + Sync + Sized,
    {
        let updated_at: DateTime<Utc> =
            crate::standard_model::delete_by_pk(ctx, Self::table_name(), self.pk()).await?;

        self.visibility_mut().deleted_at = Some(updated_at);
        self.timestamp_mut().updated_at = updated_at;

        HistoryEvent::new(
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
    async fn delete_by_id(&mut self, ctx: &DalContext) -> StandardModelResult<()>
    where
        Self: Send + Sync + Sized,
    {
        let deleted_at: DateTime<Utc> =
            crate::standard_model::delete_by_id(ctx, Self::table_name(), self.id()).await?;

        self.visibility_mut().deleted_at = Some(deleted_at);
        self.timestamp_mut().updated_at = deleted_at;

        HistoryEvent::new(
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
    async fn undelete(&mut self, ctx: &DalContext) -> StandardModelResult<()>
    where
        Self: Send + Sync + Sized,
    {
        let updated_at: DateTime<Utc> =
            crate::standard_model::undelete(ctx, Self::table_name(), self.pk()).await?;

        self.visibility_mut().deleted_at = None;
        self.timestamp_mut().updated_at = updated_at;

        HistoryEvent::new(
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

    /// Permanently delete this object from the database. This is not reversible!
    /// However, we do store the object's json representation as a HistoryEvent.
    #[instrument(skip_all)]
    async fn hard_delete(self, ctx: &DalContext) -> StandardModelResult<Self>
    where
        Self: Send + Sync + Sized + Serialize + DeserializeOwned,
    {
        let obj = crate::standard_model::hard_delete(ctx, Self::table_name(), self.pk()).await?;

        let _ = crate::HistoryEvent::new(
            ctx,
            &Self::history_event_label(vec!["hard_deleted"]),
            &Self::history_event_message("hard_deleted"),
            &serde_json::to_value(&obj)?,
        )
        .await?;

        Ok(obj)
    }

    #[instrument(skip_all)]
    async fn exists_in_head(&self, ctx: &DalContext) -> StandardModelResult<bool>
    where
        Self: Send + Sync + Sized + Serialize + DeserializeOwned,
    {
        let head_ctx = ctx.clone_with_new_visibility(Visibility::new_head(false));
        let obj: Option<Self> =
            crate::standard_model::get_by_id(&head_ctx, Self::table_name(), self.id()).await?;

        Ok(obj.is_some())
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
        impl $crate::StandardModel for $model {
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

            fn visibility(&self) -> &$crate::Visibility {
                &self.visibility
            }

            fn visibility_mut(&mut self) -> &mut $crate::Visibility {
                &mut self.visibility
            }

            fn tenancy(&self) -> &$crate::Tenancy {
                &self.tenancy
            }

            fn tenancy_mut(&mut self) -> &mut $crate::Tenancy {
                &mut self.tenancy
            }

            fn timestamp(&self) -> &$crate::Timestamp {
                &self.timestamp
            }

            fn timestamp_mut(&mut self) -> &mut $crate::Timestamp {
                &mut self.timestamp
            }
        }
    };
}
