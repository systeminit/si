use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use crate::{pk, HistoryActor, HistoryEvent, HistoryEventError, Tenancy, Timestamp};
use chrono::{DateTime, Utc};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};

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
    pub tenancy: Tenancy,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl ChangeSet {
    #[tracing::instrument(skip(txn, nats, name, note))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        note: Option<&String>,
    ) -> ChangeSetResult<Self> {
        let name = name.as_ref();
        let note = note.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM change_set_create_v1($1, $2, $3, $4)",
                &[&name, &note, &ChangeSetStatus::Open.to_string(), &tenancy],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // TODO(fnichol): determine subject(s) for publishing
        nats.publish("changeSet", &json).await?;
        let _history_event = HistoryEvent::new(
            &txn,
            &nats,
            "change_set.create",
            &history_actor,
            "Change Set created",
            &json,
            &tenancy,
        )
        .await?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    #[tracing::instrument(skip(txn, nats))]
    pub async fn apply(
        &mut self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        history_actor: &HistoryActor,
    ) -> ChangeSetResult<()> {
        let actor = serde_json::to_value(&history_actor)?;
        let row = txn
            .query_one(
                "SELECT timestamp_updated_at FROM change_set_apply_v1($1, $2)",
                &[&self.pk, &actor],
            )
            .await?;
        let updated_at: DateTime<Utc> = row.try_get("timestamp_updated_at")?;
        self.timestamp.updated_at = updated_at;
        self.status = ChangeSetStatus::Applied;
        let _history_event = HistoryEvent::new(
            &txn,
            &nats,
            "change_set.apply",
            &history_actor,
            "Change Set applied",
            &serde_json::json![{ "pk": &self.pk }],
            &self.tenancy,
        )
        .await?;
        Ok(())
    }
}
