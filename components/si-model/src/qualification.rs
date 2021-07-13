use crate::{SiChangeSet, SiStorable};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use thiserror::Error;

const QUALIFICATION_FOR_EDIT_SESSION: &str =
    include_str!("./queries/qualification_for_edit_session.sql");
const QUALIFICATION_FOR_CHANGE_SET: &str =
    include_str!("./queries/qualification_for_change_set.sql");
const QUALIFICATION_FOR_HEAD: &str = include_str!("./queries/qualification_for_head.sql");

#[derive(Error, Debug)]
pub enum QualificationError {
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("no change set when one was expected")]
    NoChangeSet,
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type QualificationResult<T> = Result<T, QualificationError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Qualification {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub qualified: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub si_storable: SiStorable,
    pub si_change_set: Option<SiChangeSet>,
}

impl Qualification {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        entity_id: impl AsRef<str>,
        name: impl AsRef<str>,
        qualified: bool,
        output: Option<String>,
        error: Option<String>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> QualificationResult<Qualification> {
        let entity_id = entity_id.as_ref();
        let name = name.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let workspace_id = workspace_id.as_ref();

        let row = txn
            .query_one(
                "SELECT object FROM qualification_create_or_update_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &entity_id,
                    &name,
                    &qualified,
                    &output,
                    &error,
                    &change_set_id,
                    &edit_session_id,
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let q: Qualification = serde_json::from_value(json)?;
        Ok(q)
    }

    pub async fn for_edit_session(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> QualificationResult<Vec<Qualification>> {
        let entity_id = entity_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let mut results: Vec<Qualification> = vec![];
        let rows = txn
            .query(
                QUALIFICATION_FOR_EDIT_SESSION,
                &[&entity_id, &change_set_id, &edit_session_id],
            )
            .await?;
        for row in rows.into_iter() {
            let object: serde_json::Value = match row.try_get("object") {
                Ok(o) => o,
                Err(_e) => continue,
            };
            let q: Qualification = serde_json::from_value(object)?;
            results.push(q);
        }
        Ok(results)
    }

    pub async fn for_change_set(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> QualificationResult<Vec<Qualification>> {
        let entity_id = entity_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let mut results: Vec<Qualification> = vec![];
        let rows = txn
            .query(QUALIFICATION_FOR_CHANGE_SET, &[&entity_id, &change_set_id])
            .await?;
        for row in rows.into_iter() {
            let object: serde_json::Value = match row.try_get("object") {
                Ok(o) => o,
                Err(_e) => continue,
            };
            let q: Qualification = serde_json::from_value(object)?;
            results.push(q);
        }
        Ok(results)
    }

    pub async fn for_head(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
    ) -> QualificationResult<Vec<Qualification>> {
        let entity_id = entity_id.as_ref();
        let mut results: Vec<Qualification> = vec![];
        let rows = txn.query(QUALIFICATION_FOR_HEAD, &[&entity_id]).await?;
        for row in rows.into_iter() {
            let object: serde_json::Value = match row.try_get("object") {
                Ok(o) => o,
                Err(_e) => continue,
            };
            let q: Qualification = serde_json::from_value(object)?;
            results.push(q);
        }
        Ok(results)
    }

    pub async fn for_head_or_change_set(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: Option<&String>,
    ) -> QualificationResult<Vec<Qualification>> {
        if let Some(change_set_id) = change_set_id {
            Qualification::for_change_set(&txn, entity_id, change_set_id).await
        } else {
            Qualification::for_head(&txn, entity_id).await
        }
    }

    pub async fn for_head_or_change_set_or_edit_session(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
    ) -> QualificationResult<Vec<Qualification>> {
        if let Some(edit_session_id) = edit_session_id {
            if let Some(change_set_id) = change_set_id {
                Qualification::for_edit_session(&txn, &entity_id, change_set_id, edit_session_id)
                    .await
            } else {
                return Err(QualificationError::NoChangeSet);
            }
        } else if let Some(change_set_id) = change_set_id {
            Qualification::for_change_set(&txn, entity_id, change_set_id).await
        } else {
            Qualification::for_head(&txn, entity_id).await
        }
    }
}
