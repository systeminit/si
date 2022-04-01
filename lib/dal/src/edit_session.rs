use crate::DalContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use strum_macros::{Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, standard_model::object_option_from_row_option, ChangeSetPk, HistoryActor, HistoryEvent,
    HistoryEventError, StandardModelError, Timestamp, WriteTenancy, WsEvent, WsEventError,
    WsPayload,
};

const EDIT_SESSION_GET_BY_PK: &str = include_str!("./queries/edit_session_get_by_pk.sql");

#[derive(Error, Debug)]
pub enum EditSessionError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standadrd model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type EditSessionResult<T> = Result<T, EditSessionError>;

#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Clone)]
pub enum EditSessionStatus {
    Open,
    Canceled,
    Saved,
}

pk!(EditSessionPk);
pk!(EditSessionId);

pub const NO_EDIT_SESSION_PK: EditSessionPk = EditSessionPk(-1);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct EditSession {
    pub pk: EditSessionPk,
    pub id: EditSessionId,
    pub name: String,
    pub note: Option<String>,
    pub status: EditSessionStatus,
    pub change_set_pk: ChangeSetPk,
    #[serde(flatten)]
    pub tenancy: WriteTenancy,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl EditSession {
    #[instrument(skip(ctx, name, note))]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        change_set_pk: &ChangeSetPk,
        name: impl AsRef<str>,
        note: Option<&String>,
    ) -> EditSessionResult<Self> {
        let name = name.as_ref();
        let note = note.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM edit_session_create_v1($1, $2, $3, $4, $5)",
                &[
                    &name,
                    &note,
                    &EditSessionStatus::Open.to_string(),
                    &change_set_pk,
                    ctx.write_tenancy(),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // TODO(fnichol): determine subject(s) for publishing
        ctx.txns().nats().publish("editSession", &json).await?;
        let _history_event =
            HistoryEvent::new(ctx, "edit_session.create", "Edit Session created", &json).await?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    #[instrument(skip(ctx))]
    pub async fn get_by_pk(
        ctx: &DalContext<'_, '_>,
        pk: &EditSessionPk,
    ) -> EditSessionResult<Option<Self>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(EDIT_SESSION_GET_BY_PK, &[ctx.read_tenancy(), &pk])
            .await?;
        let result = object_option_from_row_option(row)?;
        Ok(result)
    }

    #[instrument(skip(ctx))]
    pub async fn save(&mut self, ctx: &DalContext<'_, '_>) -> EditSessionResult<()> {
        let actor = serde_json::to_value(ctx.history_actor())?;
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT timestamp_updated_at FROM edit_session_save_v1($1, $2)",
                &[&self.pk, &actor],
            )
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("timestamp_updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = EditSessionStatus::Saved;
        let _history_event = HistoryEvent::new(
            ctx,
            "edit_session.save",
            "Edit Session saved",
            &serde_json::json![{ "pk": &self.pk }],
        )
        .await?;
        WsEvent::edit_session_saved(self, ctx.history_actor())
            .publish(ctx.txns().nats())
            .await?;
        Ok(())
    }

    #[instrument(skip(ctx))]
    pub async fn cancel(&mut self, ctx: &DalContext<'_, '_>) -> EditSessionResult<()> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT timestamp_updated_at FROM edit_session_cancel_v1($1)",
                &[&self.pk],
            )
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("timestamp_updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = EditSessionStatus::Canceled;
        let _history_event = HistoryEvent::new(
            ctx,
            "edit_session.cancel",
            "Edit Session cancelled",
            &serde_json::json![{ "pk": &self.pk }],
        )
        .await?;
        Ok(())
    }
}

impl WsEvent {
    pub fn edit_session_saved(
        edit_session: &EditSession,
        history_actor: impl Into<HistoryActor>,
    ) -> Self {
        let billing_account_ids = WsEvent::billing_account_id_from_tenancy(&edit_session.tenancy);
        let history_actor = history_actor.into();
        WsEvent::new(
            billing_account_ids,
            history_actor,
            WsPayload::EditSessionSaved(edit_session.change_set_pk),
        )
    }
}
