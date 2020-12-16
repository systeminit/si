use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{NatsTxn, NatsTxnError, PgTxn};

use crate::models::{
    list_model, next_update_clock, ListReply, ModelError, OrderByDirection, PageToken, Query,
    SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiStorable, UpdateClockError,
};

const SYSTEM_GET_ANY: &str = include_str!("../data/queries/system_get_any.sql");
const SYSTEM_GET_HEAD: &str = include_str!("../data/queries/system_get_head.sql");
const SYSTEM_GET_PROJECTION: &str = include_str!("../data/queries/system_get_projection.sql");
const SYSTEM_GET_HEAD_OR_BASE: &str = include_str!("../data/queries/system_get_head_or_base.sql");

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("no head entity found; logic error")]
    NoHead,
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("database returned a malformed entry")]
    MalformedDatabaseEntry,
    #[error("tried to save a projection without a change set")]
    MissingChangeSet,
}

pub type SystemResult<T> = Result<T, SystemError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: System,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_id: String,
    pub head: bool,
    pub base: bool,
    pub si_storable: SiStorable,
    pub si_change_set: Option<SiChangeSet>,
}

impl System {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: Option<String>,
        description: Option<String>,
        node_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> SystemResult<System> {
        let node_id = node_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let name = crate::models::generate_name(name);
        let description = if description.is_some() {
            description.unwrap()
        } else {
            name.clone()
        };

        let workspace_update_clock = next_update_clock(workspace_id).await?;
        let change_set_update_clock = next_update_clock(change_set_id).await?;

        let row = txn
            .query_one(
                "SELECT object FROM system_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    &name,
                    &description,
                    &node_id,
                    &change_set_id,
                    &edit_session_id,
                    &SiChangeSetEvent::Create.to_string(),
                    &workspace_id,
                    &workspace_update_clock.epoch,
                    &workspace_update_clock.update_count,
                    &change_set_update_clock.epoch,
                    &change_set_update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: System = serde_json::from_value(json)?;
        match object.si_change_set.as_ref() {
            Some(si_change_set) => {
                si_change_set
                    .create_change_set_participants(&txn, &nats, &object.id, &workspace_id)
                    .await?;
            }
            None => return Err(SystemError::MalformedDatabaseEntry),
        }

        Ok(object)
    }

    pub async fn save_head(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> SystemResult<()> {
        self.head = true;
        self.base = false;
        self.si_change_set = None;

        let update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = update_clock;

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM system_save_head_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: System = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn save_projection(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> SystemResult<()> {
        self.head = false;
        self.base = false;
        if self.si_change_set.is_none() {
            return Err(SystemError::MissingChangeSet);
        }

        let workspace_update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = workspace_update_clock;
        let change_set_update_clock =
            next_update_clock(&self.si_change_set.as_ref().unwrap().change_set_id).await?;
        self.si_change_set.as_mut().unwrap().order_clock = change_set_update_clock;

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM system_save_projection_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: System = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> SystemResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "systems_head",
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

    pub async fn get_any(txn: &PgTxn<'_>, id: impl AsRef<str>) -> SystemResult<System> {
        let id = id.as_ref();
        let row = txn.query_one(SYSTEM_GET_ANY, &[&id]).await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: System = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_head(txn: &PgTxn<'_>, id: impl AsRef<str>) -> SystemResult<System> {
        let id = id.as_ref();
        let row = txn.query_one(SYSTEM_GET_HEAD, &[&id]).await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: System = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_head_or_base(
        txn: &PgTxn<'_>,
        id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> SystemResult<System> {
        let id = id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let row = txn
            .query_one(SYSTEM_GET_HEAD_OR_BASE, &[&id, &change_set_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: System = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_projection(
        txn: &PgTxn<'_>,
        id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> SystemResult<System> {
        let id = id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let row = txn
            .query_one(SYSTEM_GET_PROJECTION, &[&id, &change_set_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: System = serde_json::from_value(json)?;
        Ok(object)
    }
}
