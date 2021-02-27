use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use crate::{generate_name, LabelList, LabelListItem, SiStorable};
//use crate::veritech::Veritech;
//use crate::{
//    calculate_properties, list_model, next_update_clock, ops, Edge, EdgeError, EdgeKind, Entity,
//    EntityError, Event, EventError, ListReply, ModelError, OrderByDirection, PageToken, Query,
//    SiChangeSet, SiChangeSetEvent, SiStorable, System, SystemError, UpdateClock, UpdateClockError,
//};

const CHANGE_SET_LIST_AS_LABLES: &str = include_str!("./queries/change_set_list_as_labels.sql");

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("malformed change set entry; type is missing")]
    TypeMissing,
    #[error("malformed change set entry; id is missing")]
    IdMissing,
    #[error("malformed change set entry; to_id is missing")]
    ToIdMissing,
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("missing head value in object")]
    MissingHead,
    #[error("missing change set event field")]
    EventMissing,
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiChangeSet {
    change_set_id: String,
    edit_session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ChangeSetStatus {
    Open,
    Closed,
    Abandoned,
    Applied,
    Failed,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    pub id: String,
    pub name: String,
    pub note: String,
    pub status: ChangeSetStatus,
    pub si_storable: SiStorable,
}

impl ChangeSet {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: Option<String>,
        workspace_id: String,
    ) -> ChangeSetResult<ChangeSet> {
        let name = generate_name(name);
        let row = txn
            .query_one(
                "SELECT object FROM change_set_create_v1($1, $2, $3, $4)",
                &[
                    &name,
                    &String::new(),
                    &ChangeSetStatus::Open.to_string(),
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ChangeSet = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
    ) -> ChangeSetResult<ChangeSet> {
        let id = change_set_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM change_set_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> ChangeSetResult<()> {
        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM change_set_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: ChangeSet = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn list_as_labels(
        txn: &PgTxn<'_>,
        workspace_id: impl AsRef<str>,
    ) -> ChangeSetResult<LabelList> {
        let workspace_id = workspace_id.as_ref();
        let mut results = Vec::new();
        let rows = txn
            .query(CHANGE_SET_LIST_AS_LABLES, &[&workspace_id])
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("item")?;
            let object: LabelListItem = serde_json::from_value(json)?;
            results.push(object);
        }

        return Ok(results);
    }

    pub async fn apply(&mut self, txn: &PgTxn<'_>) -> ChangeSetResult<()> {
        let row = txn
            .query_one("SELECT object FROM change_set_apply_v1($1)", &[&self.id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let change_set: ChangeSet = serde_json::from_value(json)?;
        *self = change_set;
        Ok(())
    }
}
