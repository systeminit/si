use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{insert_model, ModelError, SiStorable, SiStorableError};

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type ResourceResult<T> = Result<T, ResourceError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResourceStatus {
    Pending,
    InProgress,
    Created,
    Failed,
    Deleted,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResourceHealth {
    Ok,
    Warning,
    Error,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub si_storable: SiStorable,
    pub data: serde_json::Value,
    pub status: ResourceStatus,
    pub health: ResourceHealth,
    pub system_id: String,
    pub node_id: String,
    pub entity_id: String,
}

impl Resource {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        data: serde_json::Value,
        system_id: impl Into<String>,
        node_id: impl Into<String>,
        entity_id: impl Into<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: Option<String>,
    ) -> ResourceResult<Resource> {
        let system_id = system_id.into();
        let node_id = node_id.into();
        let entity_id = entity_id.into();
        let si_storable = SiStorable::new(
            db,
            "resource",
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let id = si_storable.object_id.clone();

        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let resource = Resource {
            id,
            data,
            system_id,
            node_id,
            entity_id,
            health: ResourceHealth::Unknown,
            status: ResourceStatus::Pending,
            unix_timestamp,
            timestamp,
            si_storable,
        };
        insert_model(db, nats, &resource.id, &resource).await?;

        Ok(resource)
    }
}
