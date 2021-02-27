use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{generate_name, ChangeSetError, SiStorable, Veritech};
use si_data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};

#[derive(Error, Debug)]
pub enum EditSessionError {
    #[error("changeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
}

pub type EditSessionResult<T> = Result<T, EditSessionError>;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EditSession {
    pub id: String,
    pub name: String,
    pub note: String,
    pub canceled: bool,
    pub saved: bool,
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
        let name = generate_name(name);
        let row = txn
            .query_one(
                "SELECT object FROM edit_session_create_v1($1, $2, $3, $4)",
                &[&name, &String::new(), &change_set_id, &workspace_id],
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

    pub async fn save_session(&mut self, txn: &PgTxn<'_>) -> EditSessionResult<()> {
        let row = txn
            .query_one(
                "SELECT object FROM edit_session_save_session_v1($1)",
                &[&self.id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let edit_session: EditSession = serde_json::from_value(json)?;
        *self = edit_session;
        Ok(())
    }

    pub async fn cancel(
        &mut self,
        _pg: &PgPool,
        txn: &PgTxn<'_>,
        _nats_conn: &NatsConn,
        nats: &NatsTxn,
        _veritech: &Veritech,
        _event_parent_id: Option<&str>,
    ) -> EditSessionResult<()> {
        let rows = txn
            .query("SELECT object FROM edit_session_revert_v1($1)", &[&self.id])
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = match row.try_get("object") {
                Ok(json) => json,
                Err(err) => {
                    dbg!(
                        "cannot get row for cancel check, probably fine; err={}",
                        err
                    );
                    continue;
                }
            };
            nats.publish(&json).await?;
        }
        //let mut change_set = ChangeSet::get(&txn, &self.change_set_id).await?;
        //change_set
        //    .execute(pg, txn, nats_conn, nats, veritech, true, event_parent_id)
        //    .await?;

        // The database query above returns ops that are to be skipped and not the updated
        // representation of this edit session. In order to maintain the "save-like" feel of
        // `cancel()`, we'll re-fetch the current database representation and update outselves in
        // place, much like a model `save()`.
        let mut updated = Self::get(&txn, &self.id).await?;
        std::mem::swap(self, &mut updated);

        Ok(())
    }
}
