use names;
use nats::asynk::Connection;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;
use thiserror::Error;
use tracing::{error, trace};
use uuid::Uuid;

use std::collections::HashMap;

use crate::data::Db;

pub mod query;
pub use query::{
    BooleanTerm, Comparison, Expression, FieldType, Item, Query, QueryError, QueryResult,
};
pub mod update;
pub use update::{websocket_run, WebsocketToken};
pub mod billing_account;
pub use billing_account::{BillingAccount, BillingAccountError, BillingAccountResult};
pub mod user;
pub use user::{LoginReply, LoginRequest, User, UserError, UserResult};
pub mod group;
pub use group::{Capability, Group, GroupError, GroupResult};
pub mod organization;
pub use organization::{Organization, OrganizationError, OrganizationResult};
pub mod workspace;
pub use workspace::{Workspace, WorkspaceError, WorkspaceResult};
pub mod node;
pub use node::{Node, NodeError, NodeKind, NodeResult};
pub mod entity;
pub use entity::{calculate_properties, Entity, EntityError, EntityResult};
pub mod ops;
pub use ops::{OpError, OpReply, OpRequest, OpResult, SiOp};
pub mod system;
pub use system::{System, SystemError, SystemResult};
pub mod edge;
pub use edge::{Edge, EdgeError, EdgeKind, EdgeResult, Vertex};
pub mod si_storable;
pub use si_storable::{
    MinimalStorable, SiStorable, SiStorableError, SiStorableResult, SimpleStorable,
};
pub mod change_set;
pub mod si_change_set;
pub use change_set::{
    ChangeSet, ChangeSetError, ChangeSetParticipant, ChangeSetResult, PatchOps, PatchReply,
    PatchRequest,
};
pub use si_change_set::{SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiChangeSetResult};
pub mod update_clock;
pub use update_clock::{UpdateClock, UpdateClockError};
pub mod edit_session;
pub use edit_session::{EditSession, EditSessionError, EditSessionResult};
pub mod jwt_key;
pub use jwt_key::{JwtKeyError, JwtKeyPrivate, JwtKeyPublic, JwtKeyResult};
pub mod page_token;
pub use page_token::{PageToken, PageTokenError, PageTokenResult};
pub mod event;
pub use event::{Event, EventError, EventKind, EventResult};
pub mod event_log;
pub use event_log::{EventLog, EventLogError, EventLogLevel, EventLogResult};
pub mod output_line;
pub use output_line::{OutputLine, OutputLineError, OutputLineResult, OutputLineStream};
pub mod resource;
pub use resource::{Resource, ResourceError, ResourceHealth, ResourceResult, ResourceStatus};
pub mod key_pair;
pub use key_pair::{KeyPairError, PublicKey};
pub mod secret;
pub use secret::{
    Secret, SecretAlgorithm, SecretError, SecretKind, SecretObjectType, SecretVersion,
};
pub mod api_client;
pub use api_client::{ApiClaim, ApiClient, ApiClientError, ApiClientResult};

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("couchbase error: {0}")]
    Couchbase(#[from] couchbase::error::CouchbaseError),
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("invalid tenancy")]
    Tenancy,
    #[error("no document found for get request with parameters")]
    GetNotFound,
    #[error("problem in serialization: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("a list model call resulted in no possible query")]
    NoQuery,
    #[error("invalid query")]
    Query(#[from] QueryError),
    #[error("page token error: {0}")]
    PageToken(#[from] PageTokenError),
    #[error("malformed list item body; missing id")]
    ListItemNoId,
    #[error("no base object found")]
    NoBase,
    #[error("object is missing a typeName")]
    MissingTypeName,
}

pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OrderByDirection {
    ASC,
    DESC,
}

impl std::fmt::Display for OrderByDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &OrderByDirection::ASC => write!(f, "ASC"),
            &OrderByDirection::DESC => write!(f, "DESC"),
        }
    }
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    pub query: Option<String>,
    pub page_size: Option<u32>,
    pub order_by: Option<String>,
    pub order_by_direction: Option<OrderByDirection>,
    pub page_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListReply {
    pub items: Vec<serde_json::Value>,
    pub total_count: u32,
    pub page_token: Option<String>,
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
pub async fn publish_model<T: Serialize + std::fmt::Debug>(
    nats: &Connection,
    model: &T,
) -> ModelResult<()> {
    let mut model_json: serde_json::Value = serde_json::to_value(model)?;
    if let Some(type_name) = model_json["siStorable"]["typeName"].as_str() {
        match type_name {
            "userPassword" | "jwtKeyPrivate" | "jwtKeyPublic" => return Ok(()),
            "keyPair" => {
                let key_pair: key_pair::KeyPair = serde_json::from_value(model_json.clone())?;
                model_json = serde_json::to_value(PublicKey::from(key_pair))?;
            }
            _ => (),
        }
    } else {
        return Err(ModelError::MissingTypeName);
    }
    let mut subject_array: Vec<String> = Vec::new();
    if let Some(tenant_ids_values) = model_json["siStorable"]["tenantIds"].as_array() {
        for tenant_id_value in tenant_ids_values.iter() {
            let tenant_id = String::from(tenant_id_value.as_str().unwrap());
            subject_array.push(tenant_id);
        }
    } else {
        match model_json["siStorable"]["billingAccountId"].as_str() {
            Some(billing_account_id) => subject_array.push(billing_account_id.into()),
            None => return Ok(()),
        }
    }
    if subject_array.len() != 0 {
        let subject: String = subject_array.join(".");
        trace!(?subject, "publishing model");
        nats.publish(&subject, model_json.to_string()).await?;
    } else {
        error!(?model, "tried to publish a model that has no tenancy!");
    }
    Ok(())
}

#[tracing::instrument(level = "trace")]
pub async fn publish_model_delete<T: Serialize + std::fmt::Debug>(
    nats: &Connection,
    model: &T,
) -> ModelResult<()> {
    let model_json: serde_json::Value = serde_json::to_value(model)?;
    if let Some(type_name) = model_json["siStorable"]["typeName"].as_str() {
        match type_name {
            "userPassword" | "jwtKeyPrivate" | "jwtKeyPublic" => return Ok(()),
            _ => (),
        }
    } else {
        return Err(ModelError::MissingTypeName);
    }
    let mut subject_array: Vec<String> = Vec::new();
    if let Some(tenant_ids_values) = model_json["siStorable"]["tenantIds"].as_array() {
        for tenant_id_value in tenant_ids_values.iter() {
            let tenant_id = String::from(tenant_id_value.as_str().unwrap());
            subject_array.push(tenant_id);
        }
    }
    if subject_array.len() != 0 {
        let subject: String = subject_array.join(".");
        trace!(?subject, "publishing model");
        nats.publish(
            &subject,
            serde_json::json![{ "deleted": model_json }].to_string(),
        )
        .await?;
    } else {
        error!(
            ?model,
            "tried to publish a model delete that has no tenancy!"
        );
    }
    Ok(())
}

#[tracing::instrument(level = "trace")]
pub async fn delete_model<T: Serialize + std::fmt::Debug>(
    db: &Db,
    nats: &Connection,
    id: impl AsRef<str> + std::fmt::Debug,
    model: &T,
) -> ModelResult<()> {
    let collection = db.bucket.default_collection();
    collection.remove(id.as_ref(), None).await?;
    publish_model_delete(nats, model).await?;
    Ok(())
}

#[tracing::instrument(level = "trace")]
pub async fn insert_model<T: Serialize + std::fmt::Debug>(
    db: &Db,
    nats: &Connection,
    id: impl AsRef<str> + std::fmt::Debug,
    model: &T,
) -> ModelResult<()> {
    let collection = db.bucket.default_collection();
    collection.insert(id.as_ref(), model, None).await?;
    publish_model(nats, model).await?;
    trace!("inserted model");
    Ok(())
}

#[tracing::instrument(level = "trace")]
pub async fn insert_model_if_missing<T: Serialize + std::fmt::Debug>(
    db: &Db,
    nats: &Connection,
    id: impl AsRef<str> + std::fmt::Debug,
    model: &T,
) -> ModelResult<bool> {
    let id = id.as_ref();
    let collection = db.bucket.default_collection();
    match collection.exists(id, None).await {
        Ok(_exists) => Ok(false),
        Err(couchbase::CouchbaseError::KeyDoesNotExist) => {
            insert_model(db, nats, id, model).await?;
            Ok(true)
        }
        Err(couchbase::CouchbaseError::Success) => {
            insert_model(db, nats, id, model).await?;
            Ok(true)
        }
        Err(err) => Err(ModelError::from(err)),
    }
}

#[tracing::instrument(level = "trace")]
pub async fn upsert_model<T: Serialize + std::fmt::Debug>(
    db: &Db,
    nats: &Connection,
    id: impl AsRef<str> + std::fmt::Debug,
    model: &T,
) -> ModelResult<()> {
    let collection = db.bucket.default_collection();
    collection.upsert(id.as_ref(), model, None).await?;
    publish_model(nats, model).await?;
    trace!("upserted model");
    Ok(())
}

#[tracing::instrument(level = "trace")]
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

pub async fn load_billing_account_model(
    db: &Db,
    billing_account_id: impl AsRef<str>,
) -> ModelResult<Vec<serde_json::Value>> {
    let billing_account_id = billing_account_id.as_ref();
    let query_string = format!(
        "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.billingAccountId = $billing_account_id 
                AND (a.siStorable.typeName = \"billingAccount\" 
                      OR a.siStorable.typeName = \"changeSetParticipant\" 
                      OR a.siStorable.typeName = \"keyPair\"
                      OR a.siStorable.typeName = \"resource\")
        ",
        bucket = db.bucket_name,
    );
    let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
    named_params.insert(
        "billing_account_id".into(),
        serde_json::json![billing_account_id],
    );
    trace!(
        ?query_string,
        ?named_params,
        "loading billing account model query"
    );
    let results: Vec<serde_json::Value> = db.query(query_string, Some(named_params)).await?;
    return Ok(results);
}

pub async fn load_data_model(
    db: &Db,
    workspace_id: String,
    update_clock: UpdateClock,
) -> ModelResult<Vec<serde_json::Value>> {
    let query_string = format!(
        "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.workspaceId = $workspace_id 
                AND a.siStorable.updateClock.epoch >= $epoch
                AND a.siStorable.updateClock.updateCount > $update_count
                AND a.siStorable.typeName != 'event'
                AND a.siStorable.typeName != 'eventLog'
                AND a.siStorable.typeName != 'outputLine'
          ORDER BY a.siChangeSet.updateClock.epoch ASC, a.siChangeSet.updateClock.updateCount ASC
        ",
        bucket = db.bucket_name,
    );
    let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
    named_params.insert("workspace_id".into(), serde_json::json![workspace_id]);
    named_params.insert("epoch".into(), serde_json::json![update_clock.epoch]);
    named_params.insert(
        "update_count".into(),
        serde_json::json![update_clock.update_count],
    );
    trace!(?query_string, ?named_params, "loading data model query");
    let results: Vec<serde_json::Value> = db.query(query_string, Some(named_params)).await?;
    return Ok(results);
}

#[tracing::instrument(level = "trace")]
pub async fn list_model(
    db: &Db,
    query: Option<Query>,
    mut page_size: Option<u32>,
    mut order_by: Option<String>,
    mut order_by_direction: Option<OrderByDirection>,
    page_token: Option<PageToken>,
    type_name: Option<String>,
    tenant_id: Option<String>,
) -> ModelResult<ListReply> {
    let mut query_items: Vec<Item> = vec![];
    let mut item_id: Option<String> = None;

    let user_query = if let Some(page_token) = page_token {
        let user_query = page_token.query;
        page_size = Some(page_token.page_size);
        order_by = Some(page_token.order_by);
        order_by_direction = Some(page_token.order_by_direction);
        item_id = Some(page_token.item_id);
        user_query.unwrap()
    } else {
        if page_size.is_none() {
            page_size = Some(10);
        }

        if order_by.is_none() {
            order_by = Some(String::from("id"));
        }

        if order_by_direction.is_none() {
            order_by_direction = Some(OrderByDirection::ASC)
        }

        // Restrict by type name if one was given
        if let Some(type_name) = type_name {
            query_items.push(Item::expression(
                "siStorable.typeName",
                &type_name,
                Comparison::Equals,
                FieldType::String,
            ));
        }

        // Restrict by tenancy if one was given
        if let Some(tenant_id) = tenant_id {
            query_items.push(Item::expression(
                "siStorable.tenantIds",
                &tenant_id,
                Comparison::Contains,
                FieldType::String,
            ));
        }

        // The users query, if one was given
        if let Some(query) = query {
            query_items.push(Item::query(query));
        }

        if query_items.len() == 0 {
            return Err(ModelError::NoQuery);
        }
        Query::new(query_items, Some(BooleanTerm::And), None)
    };

    let query_string = format!(
        "SELECT a.* FROM `{bucket}` AS a 
           WHERE {query} 
           ORDER BY a.[{order_by}] {order_by_direction}",
        bucket = db.bucket_name,
        query = user_query.as_n1ql("a")?,
        order_by = serde_json::json![order_by.as_ref().unwrap()],
        order_by_direction = order_by_direction.as_ref().unwrap(),
    );

    trace!(?query_string, "query model");
    let results: Vec<serde_json::Value> = db.query(query_string, None).await?;
    let total_count = results.len() as u32;
    trace!(?total_count, ?results, "query model results");

    if total_count <= page_size.unwrap() {
        Ok(ListReply {
            items: results,
            total_count,
            page_token: None,
        })
    } else {
        let (return_items, next_item_id) = if let Some(item_id) = item_id {
            let mut return_items: Vec<serde_json::Value> = Vec::new();
            let mut start = false;
            let mut this_page_count = 0;
            let mut next_item_id = String::new();
            for item in results.into_iter() {
                if let Some(id) = item["id"].as_str() {
                    if !start && item_id == id {
                        start = true;
                    }
                    if page_size.unwrap() == this_page_count + 1 {
                        next_item_id = String::from(id);
                        break;
                    }
                }
                if start {
                    return_items.push(item);
                    this_page_count = this_page_count + 1;
                }
            }
            (return_items, next_item_id)
        } else {
            let next_item_id = if let Some(id) = results[page_size.unwrap() as usize]["id"].as_str()
            {
                String::from(id)
            } else {
                return Err(ModelError::ListItemNoId);
            };
            let one_page = results
                .into_iter()
                .take(page_size.unwrap() as usize)
                .collect();
            (one_page, next_item_id)
        };
        let sealed_token = if next_item_id != "" {
            let page_token = PageToken {
                query: Some(user_query),
                page_size: page_size.unwrap(),
                order_by: order_by.unwrap(),
                order_by_direction: order_by_direction.unwrap(),
                item_id: next_item_id,
            };
            let sealed_token = page_token.seal(&db.page_secret_key)?;
            Some(sealed_token)
        } else {
            None
        };

        Ok(ListReply {
            items: return_items,
            total_count,
            page_token: sealed_token,
        })
    }
}

#[tracing::instrument(level = "trace")]
pub async fn get_model_change_set(
    db: &Db,
    id: impl Into<String> + std::fmt::Debug,
    type_name: impl Into<String> + std::fmt::Debug,
    billing_account_id: impl Into<String> + std::fmt::Debug,
    change_set_id: Option<String>,
) -> ModelResult<serde_json::Value> {
    let id = id.into();
    let type_name = type_name.into();
    let billing_account_id = billing_account_id.into();
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

pub async fn get_base_object(
    db: &Db,
    id: impl AsRef<str>,
    change_set_id: impl AsRef<str>,
) -> ModelResult<serde_json::Value> {
    let id = id.as_ref();
    let change_set_id = change_set_id.as_ref();
    let query = format!(
        "SELECT a.*
          FROM `{bucket}` AS a
          WHERE (a.siStorable.objectId = $id AND a.head = true) OR (a.siStorable.objectId = $id AND a.siChangeSet.changeSetId = $change_set_id)
          ORDER BY head DESC, base DESC
          LIMIT 1
        ",
        bucket = db.bucket_name
    );
    let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
    named_params.insert("id".into(), serde_json::json![id]);
    named_params.insert("change_set_id".into(), serde_json::json![change_set_id]);
    trace!(?query, ?named_params, "get base object");
    let mut query_results: Vec<serde_json::Value> =
        db.query_consistent(query, Some(named_params)).await?;
    if query_results.len() == 0 {
        Err(ModelError::NoBase)
    } else {
        let result = query_results.pop().unwrap();
        Ok(result)
    }
}
