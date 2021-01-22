use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{NatsTxn, NatsTxnError, PgTxn};
use crate::models::{
    list_model, ListReply, ModelError, OrderByDirection, PageToken, Query, SimpleStorable,
};

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub si_storable: SimpleStorable,
}

impl Workspace {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        billing_account_id: impl Into<String>,
        organization_id: impl Into<String>,
    ) -> WorkspaceResult<Workspace> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();
        let organization_id = organization_id.into();

        let row = txn
            .query_one(
                "SELECT object FROM workspace_create_v1($1, $2, $3)",
                &[&name, &billing_account_id, &organization_id],
            )
            .await?;
        let workspace_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&workspace_json).await?;
        let workspace: Workspace = serde_json::from_value(workspace_json)?;

        Ok(workspace)
    }

    pub async fn save(&self, txn: &PgTxn<'_>, nats: &NatsTxn) -> WorkspaceResult<Workspace> {
        let json = serde_json::to_value(self)?;
        let row = txn
            .query_one("SELECT object FROM workspace_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let updated = serde_json::from_value(updated_result)?;
        Ok(updated)
    }

    pub async fn get(txn: &PgTxn<'_>, workspace_id: impl AsRef<str>) -> WorkspaceResult<Workspace> {
        let id = workspace_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM workspace_get_v1($1)", &[&id])
            .await?;
        let workspace_json: serde_json::Value = row.try_get("object")?;
        let workspace = serde_json::from_value(workspace_json)?;
        Ok(workspace)
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> WorkspaceResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "workspaces",
            tenant_id,
            query,
            page_size,
            order_by,
            order_by_direction,
            page_token,
        )
        .await?;
        Ok(reply)
    }
}
