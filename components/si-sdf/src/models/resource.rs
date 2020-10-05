use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

use std::collections::HashMap;

use crate::data::{Connection, DataError, Db};
use crate::models::{insert_model, upsert_model, ModelError, SiStorable, SiStorableError};

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("data error: {0}")]
    Data(#[from] DataError),
    #[error("no resource found: {0} {1}")]
    NoResource(String, String),
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
    pub state: serde_json::Value,
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
        state: serde_json::Value,
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
            state,
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

    pub async fn get(
        db: &Db,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"resource\"
            AND a.systemId = $system_id
            AND a.entityId = $entity_id
          LIMIT 1
        ",
            bucket = db.bucket_name,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("system_id".into(), serde_json::json![system_id]);
        named_params.insert("entity_id".into(), serde_json::json![entity_id]);
        let mut query_results: Vec<Resource> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(ResourceError::NoResource(
                String::from(entity_id),
                String::from(system_id),
            ))
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get_by_node_id(
        db: &Db,
        node_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let node_id = node_id.as_ref();
        let system_id = system_id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"resource\"
            AND a.systemId = $system_id
            AND a.nodeId = $node_id
          LIMIT 1
        ",
            bucket = db.bucket_name,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("system_id".into(), serde_json::json![system_id]);
        named_params.insert("node_id".into(), serde_json::json![node_id]);
        let mut query_results: Vec<Resource> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(ResourceError::NoResource(
                String::from(node_id),
                String::from(system_id),
            ))
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn from_update_for_self(
        &mut self,
        db: &Db,
        nats: &Connection,
        state: serde_json::Value,
        status: ResourceStatus,
        health: ResourceHealth,
    ) -> ResourceResult<()> {
        self.state = state;
        self.status = status;
        self.health = health;
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        self.unix_timestamp = unix_timestamp;
        self.timestamp = timestamp;

        tracing::warn!(?self, "whats your deal");
        upsert_model(db, nats, &self.id, &self).await?;
        Ok(())
    }

    pub async fn from_update(
        db: &Db,
        nats: &Connection,
        state: serde_json::Value,
        status: ResourceStatus,
        health: ResourceHealth,
        system_id: impl Into<String>,
        node_id: impl Into<String>,
        entity_id: impl Into<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: Option<String>,
    ) -> ResourceResult<Resource> {
        let entity_id = entity_id.into();
        let system_id = system_id.into();
        let node_id = node_id.into();

        let mut resource =
            if let Ok(mut resource) = Resource::get(&db, &entity_id, &system_id).await {
                resource.state = state;
                resource
            } else {
                Resource::new(
                    db,
                    nats,
                    state,
                    system_id,
                    node_id,
                    entity_id,
                    billing_account_id,
                    organization_id,
                    workspace_id,
                    created_by_user_id,
                )
                .await?
            };
        resource.status = status;
        resource.health = health;
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        resource.unix_timestamp = unix_timestamp;
        resource.timestamp = timestamp;

        upsert_model(db, nats, &resource.id, &resource).await?;
        Ok(resource)
    }
}
