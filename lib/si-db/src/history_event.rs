use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::Timestamp;
use si_id::UserPk;

use crate::{
    Result,
    actor_view::ActorView,
    context::SiDbContext,
    tenancy::Tenancy,
    transactions::SiDbTransactions as _,
    user::User,
};

const SYSTEMINIT_EMAIL_SUFFIX: &str = "@systeminit.com";
const TEST_SYSTEMINIT_EMAIL_SUFFIX: &str = "@test.systeminit.com";

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, strum::Display, Clone, Copy, Hash)]
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

    pub async fn email(&self, ctx: &impl SiDbContext) -> Result<String> {
        Ok(match self {
            HistoryActor::SystemInit => "sally@systeminit.com".to_string(),
            HistoryActor::User(user_pk) => User::get_by_pk(ctx, *user_pk).await?.email().clone(),
        })
    }

    pub async fn email_is_systeminit(&self, ctx: &impl SiDbContext) -> Result<bool> {
        let email_as_lowercase = self.email(ctx).await?.to_lowercase();
        Ok(email_as_lowercase.ends_with(SYSTEMINIT_EMAIL_SUFFIX)
            || email_as_lowercase.ends_with(TEST_SYSTEMINIT_EMAIL_SUFFIX))
    }

    pub fn user_pk(&self) -> Option<UserPk> {
        match self {
            HistoryActor::User(pk) => Some(*pk),
            HistoryActor::SystemInit => None,
        }
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
        ctx: &impl SiDbContext,
        label: impl AsRef<str>,
        message: impl AsRef<str>,
        data: &serde_json::Value,
    ) -> Result<HistoryEvent> {
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
    pub actor: ActorView,
    pub timestamp: DateTime<Utc>,
}
