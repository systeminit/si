use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;

use crate::label_list::LabelList;
use crate::standard_model::object_option_from_row_option;
use crate::ws_event::{WsEvent, WsPayload};
use crate::{
    pk, HistoryActor, HistoryEvent, HistoryEventError, LabelListError, StandardModelError, Tenancy,
    Timestamp, WsEventError,
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
    pub tenancy: Tenancy,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl ChangeSet {
    #[instrument(skip(txn, nats, name, note))]
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
        let _history_event = HistoryEvent::new(
            txn,
            nats,
            "change_set.create",
            history_actor,
            "Change Set created",
            &json,
            tenancy,
        )
        .await?;
        let object: Self = serde_json::from_value(json)?;
        WsEvent::change_set_created(&object, history_actor)
            .publish(nats)
            .await?;
        Ok(object)
    }

    #[instrument(skip(txn, nats))]
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
            txn,
            nats,
            "change_set.apply",
            history_actor,
            "Change Set applied",
            &serde_json::json![{ "pk": &self.pk }],
            &self.tenancy,
        )
        .await?;
        WsEvent::change_set_applied(self, history_actor)
            .publish(nats)
            .await?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn list_open(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
    ) -> ChangeSetResult<LabelList<ChangeSetPk>> {
        let rows = txn.query(CHANGE_SET_OPEN_LIST, &[&tenancy]).await?;
        let results = LabelList::from_rows(rows)?;
        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn get_by_pk(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        pk: &ChangeSetPk,
    ) -> ChangeSetResult<Option<ChangeSet>> {
        let row = txn
            .query_opt(CHANGE_SET_GET_BY_PK, &[&tenancy, &pk])
            .await?;
        let change_set: Option<ChangeSet> = object_option_from_row_option(row)?;
        Ok(change_set)
    }
}

impl WsEvent {
    pub fn change_set_created(
        change_set: &ChangeSet,
        history_actor: impl Into<HistoryActor>,
    ) -> Self {
        let billing_account_ids = WsEvent::billing_account_id_from_tenancy(&change_set.tenancy);
        let history_actor = history_actor.into();
        WsEvent::new(
            billing_account_ids,
            history_actor,
            WsPayload::ChangeSetCreated(change_set.pk),
        )
    }

    pub fn change_set_applied(
        change_set: &ChangeSet,
        history_actor: impl Into<HistoryActor>,
    ) -> Self {
        let billing_account_ids = WsEvent::billing_account_id_from_tenancy(&change_set.tenancy);
        let history_actor = history_actor.into();
        WsEvent::new(
            billing_account_ids,
            history_actor,
            WsPayload::ChangeSetApplied(change_set.pk),
        )
    }

    pub fn change_set_canceled(
        change_set: &ChangeSet,
        history_actor: impl Into<HistoryActor>,
    ) -> Self {
        let billing_account_ids = WsEvent::billing_account_id_from_tenancy(&change_set.tenancy);
        let history_actor = history_actor.into();
        WsEvent::new(
            billing_account_ids,
            history_actor,
            WsPayload::ChangeSetCanceled(change_set.pk),
        )
    }
}
