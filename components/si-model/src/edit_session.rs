use crate::{generate_name, ChangeSetError, SiStorable};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditSessionError {
    #[error("changeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
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

    pub async fn cancel(&mut self, txn: &PgTxn<'_>) -> EditSessionResult<()> {
        let row = txn
            .query_one("SELECT object FROM edit_session_cancel_v1($1)", &[&self.id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let edit_session: EditSession = serde_json::from_value(json)?;
        *self = edit_session;
        Ok(())
    }
}
