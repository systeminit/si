use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{NatsTxn, NatsTxnError, PgTxn};
use crate::models::{
    list_model, ListReply, ModelError, OrderByDirection, PageToken, Query, SimpleStorable,
};

const GROUP_GET_ADMINISTRATORS_GROUP: &str =
    include_str!("../data/queries/group_get_administrators_group.sql");

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("a group with this name already exists")]
    NameExists,
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("invalid uft-8 string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("error generating password hash")]
    PasswordHash,
    #[error("group not found")]
    NotFound,
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
}

pub type GroupResult<T> = Result<T, GroupError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub subject: String,
    pub action: String,
}

impl Capability {
    pub fn new(subject: impl Into<String>, action: impl Into<String>) -> Capability {
        let subject = subject.into();
        let action = action.into();
        Capability { subject, action }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub user_ids: Vec<String>,
    pub api_client_ids: Vec<String>,
    pub capabilities: Vec<Capability>,
    pub si_storable: SimpleStorable,
}

impl Group {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        user_ids: Vec<String>,
        api_client_ids: Vec<String>,
        capabilities: Vec<Capability>,
        billing_account_id: impl Into<String>,
    ) -> GroupResult<Group> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();
        let capabilities = serde_json::to_value(capabilities)?;

        let row = txn
            .query_one(
                "SELECT object FROM group_create_v1($1, $2, $3, $4, $5)",
                &[
                    &name,
                    &user_ids,
                    &api_client_ids,
                    &capabilities,
                    &billing_account_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Group = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn get(txn: &PgTxn<'_>, group_id: impl AsRef<str>) -> GroupResult<Group> {
        let id = group_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM group_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> GroupResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "groups",
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

    pub async fn save(&self, txn: &PgTxn<'_>, nats: &NatsTxn) -> GroupResult<Group> {
        let json = serde_json::to_value(self)?;
        let row = txn
            .query_one("SELECT object FROM group_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let updated = serde_json::from_value(updated_result)?;
        Ok(updated)
    }

    pub async fn get_administrators_group(
        txn: &PgTxn<'_>,
        billing_account_id: impl AsRef<str>,
    ) -> GroupResult<Group> {
        let billing_account_id = billing_account_id.as_ref();

        let row = txn
            .query_one(GROUP_GET_ADMINISTRATORS_GROUP, &[&billing_account_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let group: Group = serde_json::from_value(json)?;
        Ok(group)
    }
}
