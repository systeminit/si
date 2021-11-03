use chrono::{DateTime, Utc};
use postgres_types::ToSql;
use serde::de::DeserializeOwned;
use serde::Serialize;
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use thiserror::Error;

use crate::{HistoryActor, HistoryEvent, HistoryEventError, Tenancy, Timestamp, Visibility};

#[derive(Error, Debug)]
pub enum StandardModelError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats error")]
    Nats(#[from] NatsError),
    #[error("{0} pk {1} is missing when one was expected; it does not exist, is not visible, or is not valid for this tenancy")]
    ModelMissing(String, String),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
}

pub type StandardModelResult<T> = Result<T, StandardModelError>;

#[tracing::instrument(skip(txn))]
pub async fn get_by_pk<PK: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    txn: &PgTxn<'_>,
    table: &str,
    pk: &PK,
) -> StandardModelResult<OBJECT> {
    let row = txn
        .query_one("SELECT object FROM get_by_pk_v1($1, $2)", &[&table, &pk])
        .await?;
    let json: serde_json::Value = row.try_get("object")?;
    let object: OBJECT = serde_json::from_value(json)?;
    Ok(object)
}

#[tracing::instrument(skip(txn))]
pub async fn get_by_id<ID: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    id: &ID,
) -> StandardModelResult<Option<OBJECT>> {
    let row_option = txn
        .query_opt(
            "SELECT * FROM get_by_id_v1($1, $2, $3, $4)",
            &[&table, &tenancy, &visibility, &id],
        )
        .await?;
    object_option_from_row_option(row_option)
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

#[tracing::instrument(skip(txn))]
pub async fn belongs_to<ID: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    retrieve_table: &str,
    id: &ID,
) -> StandardModelResult<Option<OBJECT>> {
    let row_option = txn
        .query_opt(
            "SELECT * FROM belongs_to_v1($1, $2, $3, $4, $5)",
            &[&table, &tenancy, &visibility, &retrieve_table, &id],
        )
        .await?;
    object_option_from_row_option(row_option)
}

#[tracing::instrument(skip(txn))]
pub async fn set_belongs_to<ObjectId: Send + Sync + ToSql, BelongsToId: Send + Sync + ToSql>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    object_id: &ObjectId,
    belongs_to_id: &BelongsToId,
) -> StandardModelResult<()> {
    txn.query_one(
        "SELECT set_belongs_to_v1($1, $2, $3, $4, $5)",
        &[&table, &tenancy, &visibility, &object_id, &belongs_to_id],
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip(txn))]
pub async fn unset_belongs_to<ObjectId: Send + Sync + ToSql>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    object_id: &ObjectId,
) -> StandardModelResult<()> {
    txn.query_one(
        "SELECT unset_belongs_to_v1($1, $2, $3, $4)",
        &[&table, &tenancy, &visibility, &object_id],
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip(txn))]
pub async fn has_many<ID: Send + Sync + ToSql, OBJECT: DeserializeOwned>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    retrieve_table: &str,
    belongs_to_id: &ID,
) -> StandardModelResult<Vec<OBJECT>> {
    let rows = txn
        .query(
            "SELECT * FROM has_many_v1($1, $2, $3, $4, $5)",
            &[
                &table,
                &tenancy,
                &visibility,
                &retrieve_table,
                &belongs_to_id,
            ],
        )
        .await?;
    Ok(objects_from_rows(rows)?)
}

#[tracing::instrument(skip(txn))]
pub async fn many_to_many<
    LeftId: Send + Sync + ToSql,
    RightId: Send + Sync + ToSql,
    Object: DeserializeOwned,
>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    left_table: &str,
    right_table: &str,
    left_object_id: Option<&LeftId>,
    right_object_id: Option<&RightId>,
) -> StandardModelResult<Vec<Object>> {
    let rows = txn
        .query(
            "SELECT * FROM many_to_many_v1($1, $2, $3, $4, $5, $6, $7)",
            &[
                &table,
                &tenancy,
                &visibility,
                &left_table,
                &right_table,
                &left_object_id,
                &right_object_id,
            ],
        )
        .await?;
    Ok(objects_from_rows(rows)?)
}

#[tracing::instrument(skip(txn))]
pub async fn associate_many_to_many<LeftId: Send + Sync + ToSql, RightId: Send + Sync + ToSql>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    left_object_id: &LeftId,
    right_object_id: &RightId,
) -> StandardModelResult<()> {
    txn.query_one(
        "SELECT associate_many_to_many_v1($1, $2, $3, $4, $5)",
        &[
            &table,
            &tenancy,
            &visibility,
            &left_object_id,
            &right_object_id,
        ],
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip(txn))]
pub async fn disassociate_many_to_many<
    LeftId: Send + Sync + ToSql,
    RightId: Send + Sync + ToSql,
>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
    left_object_id: &LeftId,
    right_object_id: &RightId,
) -> StandardModelResult<()> {
    txn.query_one(
        "SELECT disassociate_many_to_many_v1($1, $2, $3, $4, $5)",
        &[
            &table,
            &tenancy,
            &visibility,
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

#[tracing::instrument(skip(txn))]
pub async fn update<PK: Send + Sync + ToSql + std::fmt::Display, VALUE: Send + Sync + ToSql>(
    txn: &PgTxn<'_>,
    table: &str,
    column: &str,
    tenancy: &Tenancy,
    pk: &PK,
    value: &VALUE,
) -> StandardModelResult<DateTime<Utc>> {
    let row = txn
        .query_one(
            "SELECT updated_at FROM update_by_pk_v1($1, $2, $3, $4, $5)",
            &[&table, &column, &tenancy, &pk, &value],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), pk.to_string()))
}

#[tracing::instrument(skip(txn))]
pub async fn list<OBJECT: DeserializeOwned>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    visibility: &Visibility,
) -> StandardModelResult<Vec<OBJECT>> {
    let rows = txn
        .query(
            "SELECT * FROM list_models_v1($1, $2, $3)",
            &[&table, &tenancy, &visibility],
        )
        .await?;
    Ok(objects_from_rows(rows)?)
}

#[tracing::instrument(skip(txn))]
pub async fn delete<PK: Send + Sync + ToSql + std::fmt::Display>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    pk: PK,
) -> StandardModelResult<DateTime<Utc>> {
    let row = txn
        .query_one(
            "SELECT updated_at FROM delete_by_pk_v1($1, $2, $3)",
            &[&table, &tenancy, &pk],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), pk.to_string()))
}

#[tracing::instrument(skip(txn))]
pub async fn undelete<PK: Send + Sync + ToSql + std::fmt::Display>(
    txn: &PgTxn<'_>,
    table: &str,
    tenancy: &Tenancy,
    pk: PK,
) -> StandardModelResult<DateTime<Utc>> {
    let row = txn
        .query_one(
            "SELECT updated_at FROM undelete_by_pk_v1($1, $2, $3)",
            &[&table, &tenancy, &pk],
        )
        .await?;
    row.try_get("updated_at")
        .map_err(|_| StandardModelError::ModelMissing(table.to_string(), pk.to_string()))
}

#[tracing::instrument(skip(txn, row))]
pub async fn finish_create_from_row<Object: Send + Sync + DeserializeOwned + StandardModel>(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    row: tokio_postgres::Row,
) -> StandardModelResult<Object> {
    let json: serde_json::Value = row.try_get("object")?;
    // TODO(fnichol): determine subject(s) for publishing
    nats.publish("standardModel", &json).await?;
    let _history_event = HistoryEvent::new(
        &txn,
        &nats,
        Object::history_event_label(vec!["create"]),
        &history_actor,
        Object::history_event_message("created"),
        &serde_json::json![{ "visibility": &visibility }],
        &tenancy,
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

    #[tracing::instrument(skip(txn))]
    async fn get_by_pk(txn: &PgTxn<'_>, pk: &Self::Pk) -> StandardModelResult<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let object = crate::standard_model::get_by_pk(&txn, Self::table_name(), &pk).await?;
        Ok(object)
    }

    #[tracing::instrument(skip(txn))]
    async fn get_by_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &Self::Id,
    ) -> StandardModelResult<Option<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let object =
            crate::standard_model::get_by_id(&txn, Self::table_name(), &tenancy, &visibility, &id)
                .await?;
        Ok(object)
    }

    #[tracing::instrument(skip(txn))]
    async fn list(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
    ) -> StandardModelResult<Vec<Self>>
    where
        Self: Sized + DeserializeOwned,
    {
        let result =
            crate::standard_model::list(&txn, Self::table_name(), &tenancy, &visibility).await?;
        Ok(result)
    }

    #[tracing::instrument(skip(txn, self))]
    async fn delete(
        &mut self,
        txn: &si_data::PgTxn<'_>,
        nats: &si_data::NatsTxn,
        history_actor: &crate::HistoryActor,
    ) -> StandardModelResult<()>
    where
        Self: Send + Sync,
    {
        let updated_at: chrono::DateTime<chrono::Utc> =
            crate::standard_model::delete(&txn, Self::table_name(), self.tenancy(), self.pk())
                .await?;
        self.visibility_mut().deleted = true;
        self.timestamp_mut().updated_at = updated_at;
        let _history_event = crate::HistoryEvent::new(
            &txn,
            &nats,
            &Self::history_event_label(vec!["deleted"]),
            &history_actor,
            &Self::history_event_message("deleted"),
            &serde_json::json![{ "pk": self.pk(), "id": self.id(), "visibility": self.visibility() }],
            &self.tenancy(),
        )
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(txn, self))]
    async fn undelete(
        &mut self,
        txn: &PgTxn<'_>,
        nats: &si_data::NatsTxn,
        history_actor: &crate::HistoryActor,
    ) -> StandardModelResult<()>
    where
        Self: Send + Sync,
    {
        let updated_at: chrono::DateTime<chrono::Utc> =
            crate::standard_model::undelete(&txn, Self::table_name(), self.tenancy(), self.pk())
                .await?;
        self.visibility_mut().deleted = false;
        self.timestamp_mut().updated_at = updated_at;
        let _history_event = crate::HistoryEvent::new(
            &txn,
            &nats,
            &Self::history_event_label(vec!["undeleted"]),
            &history_actor,
            &Self::history_event_message("undeleted"),
            &serde_json::json![{ "pk": self.pk(), "id": self.id(), "visibility": self.visibility() }],
            &self.tenancy(),
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

            fn tenancy(&self) -> &Tenancy {
                &self.tenancy
            }

            fn tenancy_mut(&mut self) -> &mut Tenancy {
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
