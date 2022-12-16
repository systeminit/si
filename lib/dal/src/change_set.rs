use crate::{DalContext, WsEventResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::label_list::LabelList;
use crate::standard_model::object_option_from_row_option;
use crate::ws_event::{WsEvent, WsEventError, WsPayload};
use crate::{
    job::definition::Confirmations, pk, HistoryEvent, HistoryEventError, LabelListError,
    StandardModelError, Timestamp, WriteTenancy,
};

const CHANGE_SET_OPEN_LIST: &str = include_str!("./queries/change_set_open_list.sql");
const CHANGE_SET_GET_BY_PK: &str = include_str!("./queries/change_set_get_by_pk.sql");

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    LabelList(#[from] LabelListError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct ChangeSet {
    pub pk: ChangeSetPk,
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
        ctx: &DalContext,
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
        WsEvent::change_set_created(ctx, object.pk)
            .await?
            .publish(ctx)
            .await?;
        Ok(object)
    }

    #[instrument(skip(ctx))]
    pub async fn apply(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
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

        ctx.enqueue_job(Confirmations::new(ctx)).await;

        WsEvent::change_set_applied(ctx, self.pk)
            .await?
            .publish(ctx)
            .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn list_open(ctx: &DalContext) -> ChangeSetResult<LabelList<ChangeSetPk>> {
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
        ctx: &DalContext,
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
    pub async fn change_set_created(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCreated(change_set_pk)).await
    }

    pub async fn change_set_applied(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetApplied(change_set_pk)).await
    }

    pub async fn change_set_canceled(
        ctx: &DalContext,
        change_set_pk: ChangeSetPk,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCanceled(change_set_pk)).await
    }

    pub async fn change_set_written(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetWritten(ctx.visibility().change_set_pk),
        )
        .await
    }
}
