use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use strum_macros::Display;
use thiserror::Error;

use crate::{EdgeError, Entity, Node, SiChangeSet, SiStorable};

const RESOURCES_FOR_EDIT_SESSION: &str = include_str!("./queries/resources_for_edit_session.sql");

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("no resource found: {0} {1}")]
    NoResource(String, String),
    #[error("missing change set id on resource projection save")]
    MissingChangeSetId,
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("entity error: {0}")]
    Entity(String),
    #[error("node error: {0}")]
    Node(String),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncPredecessor {
    pub entity: Entity,
    pub resource: Resource,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceRequest<'a> {
    pub system_id: &'a str,
    pub node: &'a Node,
    pub entity: &'a Entity,
    pub resource: &'a Resource,
    pub predecessors: Vec<VeritechSyncPredecessor>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceUpdate {
    pub state: serde_json::Value,
    pub status: ResourceStatus,
    pub health: ResourceHealth,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceReply {
    pub resource: VeritechSyncResourceUpdate,
}

pub type ResourceResult<T> = Result<T, ResourceError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResourceStatus {
    Pending,
    InProgress,
    Created,
    Failed,
    Deleted,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResourceHealth {
    Ok,
    Warning,
    Error,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub state: serde_json::Value,
    pub status: ResourceStatus,
    pub health: ResourceHealth,
    pub system_id: String,
    pub node_id: String,
    pub entity_id: String,
    pub si_change_set: Option<SiChangeSet>,
    pub si_storable: SiStorable,
}

impl Resource {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        state: serde_json::Value,
        system_id: impl AsRef<str>,
        node_id: impl AsRef<str>,
        entity_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let system_id = system_id.as_ref();
        let node_id = node_id.as_ref();
        let entity_id = entity_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let row = txn
            .query_one(
                "SELECT object FROM resource_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    &state,
                    &ResourceStatus::Pending.to_string(),
                    &ResourceHealth::Unknown.to_string(),
                    &timestamp,
                    &unix_timestamp,
                    &system_id,
                    &node_id,
                    &entity_id,
                    &change_set_id,
                    &edit_session_id,
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;

        let object: Resource = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn for_edit_session_by_entity_id(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> ResourceResult<Vec<Resource>> {
        let entity_id = entity_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let rows = txn
            .query(
                RESOURCES_FOR_EDIT_SESSION,
                &[&entity_id, &change_set_id, &edit_session_id],
            )
            .await?;

        let mut results: Vec<Resource> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let resource: Resource = serde_json::from_value(json)?;
            results.push(resource);
        }

        Ok(results)
    }
}
