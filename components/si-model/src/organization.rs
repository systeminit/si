use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::SimpleStorable;
use si_data::{NatsTxn, NatsTxnError, PgTxn};

#[derive(Error, Debug)]
pub enum OrganizationError {
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
}

pub type OrganizationResult<T> = Result<T, OrganizationError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub si_storable: SimpleStorable,
}

impl Organization {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        billing_account_id: impl Into<String>,
    ) -> OrganizationResult<Organization> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();

        let row = txn
            .query_one(
                "SELECT object FROM organization_create_v1($1, $2)",
                &[&name, &billing_account_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Organization = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        organization_id: impl AsRef<str>,
    ) -> OrganizationResult<Organization> {
        let id = organization_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM organization_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn save(&self, txn: &PgTxn<'_>, nats: &NatsTxn) -> OrganizationResult<Organization> {
        let json = serde_json::to_value(self)?;
        let row = txn
            .query_one("SELECT object FROM organization_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let updated = serde_json::from_value(updated_result)?;
        Ok(updated)
    }
}
