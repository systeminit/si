use names;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;
use thiserror::Error;
use uuid::Uuid;

use si_data::DataQuery;

use std::collections::HashMap;

use crate::data::Db;

pub mod billing_account;
pub use billing_account::{BillingAccount, BillingAccountError, BillingAccountResult};
pub mod user;
pub use user::{User, UserError, UserResult};
pub mod group;
pub use group::{Capability, Group, GroupError, GroupResult};
pub mod organization;
pub use organization::{Organization, OrganizationError, OrganizationResult};
pub mod workspace;
pub use workspace::{Workspace, WorkspaceError, WorkspaceResult};
pub mod node;
pub use node::{Node, NodeError, NodeKind, NodeResult};
pub mod entity;
pub use entity::ops::{OpError, OpResult};
pub use entity::{Entity, EntityError, EntityResult};
pub mod si_storable;
pub use si_storable::{SiStorable, SiStorableError, SiStorableResult, SimpleStorable};
pub mod change_set;
pub mod si_change_set;
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetResult};
pub use si_change_set::{SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiChangeSetResult};
pub mod update_clock;
pub use update_clock::{UpdateClock, UpdateClockError};
pub mod edit_session;
pub use edit_session::{EditSession, EditSessionError, EditSessionResult};

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("couchbase error: {0}")]
    Couchbase(#[from] couchbase::error::CouchbaseError),
    #[error("data layer error: {0}")]
    Data(#[from] si_data::DataError),
    #[error("invalid tenancy")]
    Tenancy,
    #[error("no document found for get request with parameters")]
    GetNotFound,
    #[error("problem in serialization: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Deserialize, Serialize, Debug)]
pub enum OrderByDirection {
    ASC,
    DSC,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub query: Option<DataQuery>,
    pub page_size: Option<u32>,
    pub order_by: Option<String>,
    pub order_by_direction: Option<OrderByDirection>,
    pub page_token: Option<String>,
    pub scope_by_tenant_id: Option<String>,
    pub type_name: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ListResponse {
    pub items: Vec<serde_json::Value>,
    pub total_count: u32,
    pub next_item_id: String,
    pub page_token: String,
}

#[derive(Deserialize, Debug)]
pub struct SetField {
    pub pointer: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub change_set_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetReply {
    pub item: serde_json::Value,
}

pub fn generate_name(name: Option<String>) -> String {
    if name.is_some() {
        return name.unwrap();
    }
    let mut name_generator = names::Generator::with_naming(names::Name::Numbered);
    let name = name_generator.next().unwrap();
    return name;
}

pub fn generate_id(type_name: impl AsRef<str>) -> String {
    let uuid = Uuid::new_v4();
    format!("{}:{}", type_name.as_ref(), uuid.to_simple().to_string())
}

#[tracing::instrument(level = "trace")]
pub async fn insert_model<T: Serialize + std::fmt::Debug>(
    db: &Db,
    id: impl AsRef<str> + std::fmt::Debug,
    model: &T,
) -> ModelResult<()> {
    let collection = db.bucket.default_collection();
    collection.insert(id.as_ref(), model, None).await?;
    tracing::trace!("inserted model");
    Ok(())
}

#[tracing::instrument(level = "trace")]
pub async fn upsert_model<T: Serialize + std::fmt::Debug>(
    db: &Db,
    id: impl AsRef<str> + std::fmt::Debug,
    model: &T,
) -> ModelResult<()> {
    let collection = db.bucket.default_collection();
    collection.upsert(id.as_ref(), model, None).await?;
    tracing::trace!("upserted model");
    Ok(())
}

pub fn check_tenancy(object: &serde_json::Value, id: impl Into<String>) -> ModelResult<()> {
    let id = id.into();
    if !(object["siStorable"]["tenantIds"].is_array()
        && object["siStorable"]["tenantIds"]
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String(id)))
    {
        Err(ModelError::Tenancy)
    } else {
        Ok(())
    }
}

pub async fn get_model<T: DeserializeOwned + std::fmt::Debug>(
    db: &Db,
    id: impl AsRef<str> + std::fmt::Debug,
    billing_account_id: impl AsRef<str> + std::fmt::Debug,
) -> ModelResult<T> {
    let id = id.as_ref();
    let billing_account_id = billing_account_id.as_ref();
    let collection = db.bucket.default_collection();
    let response = collection.get(id, None).await?;
    let json_response: serde_json::Value = response.content_as()?;
    check_tenancy(&json_response, billing_account_id)?;
    let object: T = serde_json::from_value(json_response)?;
    Ok(object)
}

#[tracing::instrument(level = "trace")]
pub async fn get_model_change_set(
    db: &Db,
    id: impl Into<String> + std::fmt::Debug,
    type_name: impl Into<String> + std::fmt::Debug,
    billing_account_id: String,
    change_set_id: Option<String>,
) -> ModelResult<serde_json::Value> {
    let id = id.into();
    let type_name = type_name.into();
    if change_set_id.is_none() {
        let collection = db.bucket.default_collection();
        let response = collection.get(&id, None).await?;
        let json_response: serde_json::Value = response.content_as()?;
        check_tenancy(&json_response, &billing_account_id)?;
        Ok(json_response)
    } else {
        let query = format!(
            "SELECT a.* 
               FROM `{bucket}` AS a 
               WHERE a.siStorable.typeName = $type_name 
                 AND a.siChangeSet.changeSetId = $change_set_id
                 AND a.siStorable.object_id = $id",
            bucket = db.bucket_name
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("type_name".into(), serde_json::json![type_name]);
        // Safe, because we checked it.
        named_params.insert(
            "change_set_id".into(),
            serde_json::json![change_set_id.unwrap()],
        );
        named_params.insert("id".into(), serde_json::json![id]);
        named_params.insert("bucket".into(), serde_json::json![db.bucket_name.as_ref()]);
        let mut query_results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(ModelError::GetNotFound)
        } else {
            let result = query_results.pop().unwrap();
            check_tenancy(&result, &billing_account_id)?;
            Ok(result)
        }
    }
}

pub async fn check_secondary_key_universal(
    db: &Db,
    type_name: impl AsRef<str>,
    key: impl AsRef<str>,
    value: impl AsRef<str>,
) -> ModelResult<bool> {
    let key = key.as_ref();
    let value = value.as_ref();
    let type_name = type_name.as_ref();

    let query = format!(
        "SELECT a.*
               FROM `{bucket}` AS a 
               WHERE a.siStorable.typeName = $type_name 
                 AND a.{key} = $value
               LIMIT 1
                 ",
        bucket = db.bucket_name,
        key = key,
    );
    let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
    named_params.insert("type_name".into(), serde_json::json![type_name]);
    named_params.insert("value".into(), serde_json::json![value]);
    let query_results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
    if query_results.len() == 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn check_secondary_key(
    db: &Db,
    tenant_id: impl AsRef<str>,
    type_name: impl AsRef<str>,
    key: impl AsRef<str>,
    value: impl AsRef<str>,
) -> ModelResult<bool> {
    let key = key.as_ref();
    let tenant_id = tenant_id.as_ref();
    let value = value.as_ref();
    let type_name = type_name.as_ref();

    let query = format!(
        "SELECT a.*
               FROM `{bucket}` AS a 
               WHERE a.siStorable.typeName = $type_name 
                 AND ARRAY_CONTAINS(a.siStorable.tenantIds, $tenant_id)
                 AND a.{key} = $value
               LIMIT 1
                 ",
        bucket = db.bucket_name,
        key = key,
    );
    let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
    named_params.insert("type_name".into(), serde_json::json![type_name]);
    named_params.insert("tenant_id".into(), serde_json::json![tenant_id]);
    named_params.insert("value".into(), serde_json::json![value]);
    let query_results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
    if query_results.len() == 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}
