use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use strum::Display as StrumDisplay;
use telemetry::prelude::*;
use thiserror::Error;

use crate::actor_view::ActorView;
use crate::{DalContext, Timestamp, User, UserPk};
use crate::{Tenancy, TransactionsError};

const SYSTEMINIT_EMAIL_SUFFIX: &str = "@systeminit.com";
const TEST_SYSTEMINIT_EMAIL_SUFFIX: &str = "@test.systeminit.com";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum HistoryEventError {
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(String),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("user error: {0}")]
    User(String),
}

pub type HistoryEventResult<T> = Result<T>;

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, StrumDisplay, Clone, Copy, Hash)]
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

    pub async fn email(&self, ctx: &DalContext) -> HistoryEventResult<String> {
        Ok(match self {
            HistoryActor::SystemInit => "sally@systeminit.com".to_string(),
            HistoryActor::User(user_pk) => User::get_by_pk_or_error(ctx, *user_pk)
                .await
                .map_err(|e| HistoryEventError::User(e.to_string()))?
                .email()
                .clone(),
        })
    }

    pub async fn email_is_systeminit(&self, ctx: &DalContext) -> HistoryEventResult<bool> {
        let email_as_lowercase = self.email(ctx).await?.to_lowercase();
        Ok(email_as_lowercase.ends_with(SYSTEMINIT_EMAIL_SUFFIX)
            || email_as_lowercase.ends_with(TEST_SYSTEMINIT_EMAIL_SUFFIX))
    }
}

impl From<UserPk> for HistoryActor {
    fn from(pk: UserPk) -> Self {
        HistoryActor::User(pk)
    }
}

pub use si_id::HistoryEventPk;

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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEventMetadata {
    pub(crate) actor: ActorView,
    pub(crate) timestamp: DateTime<Utc>,
}

impl HistoryEventMetadata {
    pub async fn from_history_actor_timestamp(
        ctx: &DalContext,
        value: HistoryActorTimestamp,
    ) -> HistoryEventResult<Self> {
        let actor = ActorView::from_history_actor(ctx, value.actor)
            .await
            .map_err(|e| HistoryEventError::StandardModel(e.to_string()))?;

        Ok(Self {
            actor,
            timestamp: value.timestamp,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryActorTimestamp {
    pub actor: HistoryActor,
    pub timestamp: DateTime<Utc>,
}
