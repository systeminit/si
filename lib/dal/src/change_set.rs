use crate::DalContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use si_data::{NatsError, PgError};
use telemetry::prelude::*;

use crate::label_list::LabelList;
use crate::standard_model::object_option_from_row_option;
use crate::ws_event::{WsEvent, WsPayload};
use crate::{
    pk, HistoryEvent, HistoryEventError, LabelListError, StandardModelError, Timestamp,
    WriteTenancy, WsEventError,
};

const CHANGE_SET_OPEN_LIST: &str = include_str!("./queries/change_set_open_list.sql");
const CHANGE_SET_GET_BY_PK: &str = include_str!("./queries/change_set_get_by_pk.sql");

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("label list error: {0}")]
    LabelList(#[from] LabelListError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq)]
pub enum ChangeSetStatus {
    Open,
    Closed,
    Abandoned,
    Applied,
    Failed,
}

pk!(ChangeSetPk);
pk!(ChangeSetId);

pub const NO_CHANGE_SET_PK: ChangeSetPk = ChangeSetPk(-1);

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct ChangeSet {
    pub pk: ChangeSetPk,
    pub id: ChangeSetId,
    pub name: String,
    pub note: Option<String>,
    pub status: ChangeSetStatus,
    #[serde(flatten)]
    pub tenancy: WriteTenancy,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl ChangeSet {
    #[instrument(skip(ctx, name, note))]
    pub async fn new(
        ctx: &DalContext<'_, '_, '_>,
        name: impl AsRef<str>,
        note: Option<&String>,
    ) -> ChangeSetResult<Self> {
        let name = name.as_ref();
        let note = note.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM change_set_create_v1($1, $2, $3, $4)",
                &[
                    &name,
                    &note,
                    &ChangeSetStatus::Open.to_string(),
                    ctx.write_tenancy(),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let _history_event =
            HistoryEvent::new(ctx, "change_set.create", "Change Set created", &json).await?;
        let object: Self = serde_json::from_value(json)?;
        WsEvent::change_set_created(ctx, &object)
            .publish(ctx)
            .await?;
        Ok(object)
    }

    #[instrument(skip(ctx))]
    pub async fn apply(&mut self, ctx: &DalContext<'_, '_, '_>) -> ChangeSetResult<()> {
        let actor = serde_json::to_value(ctx.history_actor())?;
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT timestamp_updated_at FROM change_set_apply_v1($1, $2)",
                &[&self.pk, &actor],
            )
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("timestamp_updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::Applied;
        let _history_event = HistoryEvent::new(
            ctx,
            "change_set.apply",
            "Change Set applied",
            &serde_json::json![{ "pk": &self.pk }],
        )
        .await?;
        WsEvent::change_set_applied(ctx, self).publish(ctx).await?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn list_open(
        ctx: &DalContext<'_, '_, '_>,
    ) -> ChangeSetResult<LabelList<ChangeSetPk>> {
        let rows = ctx
            .txns()
            .pg()
            .query(CHANGE_SET_OPEN_LIST, &[ctx.read_tenancy()])
            .await?;
        let results = LabelList::from_rows(rows)?;
        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn get_by_pk(
        ctx: &DalContext<'_, '_, '_>,
        pk: &ChangeSetPk,
    ) -> ChangeSetResult<Option<ChangeSet>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(CHANGE_SET_GET_BY_PK, &[ctx.read_tenancy(), &pk])
            .await?;
        let change_set: Option<ChangeSet> = object_option_from_row_option(row)?;
        Ok(change_set)
    }
}

impl WsEvent {
    pub fn change_set_created(ctx: &DalContext<'_, '_, '_>, change_set: &ChangeSet) -> Self {
        WsEvent::new(ctx, WsPayload::ChangeSetCreated(change_set.pk))
    }

    pub fn change_set_applied(ctx: &DalContext<'_, '_, '_>, change_set: &ChangeSet) -> Self {
        WsEvent::new(ctx, WsPayload::ChangeSetApplied(change_set.pk))
    }

    pub fn change_set_canceled(ctx: &DalContext<'_, '_, '_>, change_set: &ChangeSet) -> Self {
        WsEvent::new(ctx, WsPayload::ChangeSetCanceled(change_set.pk))
    }

    pub fn change_set_written(ctx: &DalContext<'_, '_, '_>) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetWritten(ctx.visibility().change_set_pk),
        )
    }
}
