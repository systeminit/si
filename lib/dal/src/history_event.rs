use crate::WriteTenancy;
use serde::{Deserialize, Serialize};
use strum_macros::Display as StrumDisplay;
use thiserror::Error;

use si_data::{NatsError, PgError};
use telemetry::prelude::*;

use crate::{pk, DalContext, Timestamp, UserId};

#[derive(Error, Debug)]
pub enum HistoryEventError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
}

pub type HistoryEventResult<T> = Result<T, HistoryEventError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, StrumDisplay, Clone)]
pub enum HistoryActor {
    User(UserId),
    SystemInit,
}

impl From<UserId> for HistoryActor {
    fn from(id: UserId) -> Self {
        HistoryActor::User(id)
    }
}

impl From<&HistoryActor> for HistoryActor {
    fn from(history_actor: &HistoryActor) -> Self {
        history_actor.clone()
    }
}

pk!(HistoryEventPk);

/// HistoryEvents are the audit trail for things in SI. They track
/// that a specific actor did something, and optionally store data
/// associated with the activity for posterity.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct HistoryEvent {
    pub pk: HistoryEventPk,
    pub label: String,
    pub actor: HistoryActor,
    pub message: String,
    pub data: serde_json::Value,
    #[serde(flatten)]
    pub tenancy: WriteTenancy,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl HistoryEvent {
    #[instrument(skip(ctx, label, message))]
    pub async fn new(
        ctx: &DalContext<'_, '_, '_>,
        label: impl AsRef<str>,
        message: impl AsRef<str>,
        data: &serde_json::Value,
    ) -> HistoryEventResult<HistoryEvent> {
        let label = label.as_ref();
        let message = message.as_ref();
        let actor = serde_json::to_value(ctx.history_actor())?;
        let txns = ctx.txns();
        let row = txns
            .pg()
            .query_one(
                "SELECT object FROM history_event_create_v1($1, $2, $3, $4, $5)",
                &[
                    &label.to_string(),
                    &actor,
                    &message,
                    &data,
                    ctx.write_tenancy(),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // TODO(fnichol): determine subject(s) for publishing
        txns.nats().publish("historyEvent", &json).await?;
        let object: HistoryEvent = serde_json::from_value(json)?;
        Ok(object)
    }
}
