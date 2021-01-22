use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};
use crate::models::{
    next_update_clock, ChangeSet, ChangeSetError, ModelError, SiStorable, UpdateClockError,
};
use crate::veritech::Veritech;

#[derive(Error, Debug)]
pub enum EditSessionError {
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("changeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
}

pub type EditSessionResult<T> = Result<T, EditSessionError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchRequest {
    Cancel(bool),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchReply {
    Cancel(EditSession),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
    pub workspace_id: String,
    pub organization_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: EditSession,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditSession {
    pub id: String,
    pub name: String,
    pub note: String,
    pub reverted: bool,
    pub change_set_id: String,
    pub si_storable: SiStorable,
}

impl EditSession {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: Option<String>,
        change_set_id: String,
        workspace_id: String,
    ) -> EditSessionResult<EditSession> {
        let name = crate::models::generate_name(name);
        let update_clock = next_update_clock(&workspace_id).await?;
        let row = txn
            .query_one(
                "SELECT object FROM edit_session_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    &name,
                    &String::new(),
                    &change_set_id,
                    &workspace_id,
                    &update_clock.epoch,
                    &update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: EditSession = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        edit_session_id: impl AsRef<str>,
    ) -> EditSessionResult<EditSession> {
        let id = edit_session_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM edit_session_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn cancel(
        &self,
        pg: &PgPool,
        txn: &PgTxn<'_>,
        nats_conn: &NatsConn,
        nats: &NatsTxn,
        veritech: &Veritech,
        event_parent_id: Option<&str>,
    ) -> EditSessionResult<()> {
        let rows = txn
            .query("SELECT object FROM edit_session_revert_v1($1)", &[&self.id])
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = match row.try_get("object") {
                Ok(json) => json,
                Err(e) => {
                    dbg!("cannot get row for cancel check, probably fine");
                    dbg!(&e);
                    continue;
                }
            };
            nats.publish(&json).await?;
        }
        let mut change_set = ChangeSet::get(&txn, &self.change_set_id).await?;
        change_set
            .execute(pg, txn, nats_conn, nats, veritech, true, event_parent_id)
            .await?;

        Ok(())
    }
}
