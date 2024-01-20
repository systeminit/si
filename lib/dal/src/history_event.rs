use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use strum::Display as StrumDisplay;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{pk, DalContext, Timestamp, UserPk};
use crate::{Tenancy, TransactionsError};

pub use metadata::HistoryEventMetadata;

mod metadata;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum HistoryEventError {
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type HistoryEventResult<T> = Result<T, HistoryEventError>;

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, StrumDisplay, Clone, Copy)]
pub enum HistoryActor {
    SystemInit,
    User(UserPk),
}

impl HistoryActor {
    pub fn distinct_id(&self) -> String {
        match self {
            HistoryActor::User(pk) => pk.to_string(),
            HistoryActor::SystemInit => "unknown-backend".to_string(),
        }
    }
}

impl From<UserPk> for HistoryActor {
    fn from(pk: UserPk) -> Self {
        HistoryActor::User(pk)
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
    pub tenancy: Tenancy,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl HistoryEvent {
    #[instrument(skip(ctx, label, message))]
    pub async fn new(
        ctx: &DalContext,
        label: impl AsRef<str>,
        message: impl AsRef<str>,
        data: &serde_json::Value,
    ) -> HistoryEventResult<HistoryEvent> {
        let label = label.as_ref();
        let message = message.as_ref();
        let actor = serde_json::to_value(ctx.history_actor())?;
        let txns = ctx.txns().await?;
        let row = txns
            .pg()
            .query_one(
                "SELECT object FROM history_event_create_v1($1, $2, $3, $4, $5)",
                &[&label.to_string(), &actor, &message, &data, ctx.tenancy()],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // TODO(fnichol): determine subject(s) for publishing
        txns.nats().publish("historyEvent", &json).await?;
        let object: HistoryEvent = serde_json::from_value(json)?;
        Ok(object)
    }
}
