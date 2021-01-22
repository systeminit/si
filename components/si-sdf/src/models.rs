use names;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod si_storable;
pub use si_storable::{MinimalStorable, SiStorable, SimpleStorable};
pub mod workspace;
pub use workspace::{Workspace, WorkspaceError};
pub mod organization;
pub use organization::{Organization, OrganizationError};
pub mod query;
pub use query::{
    BooleanTerm, Comparison, Expression, FieldType, Item, Query, QueryError, QueryResult,
};
pub mod page_token;
pub use page_token::{PageToken, PageTokenError, PageTokenResult};
pub mod user;
pub use user::{LoginReply, LoginRequest, User, UserError, UserResult};
pub mod group;
pub use group::{Capability, Group, GroupError, GroupResult};
pub mod key_pair;
pub use key_pair::{KeyPair, KeyPairError, PublicKey};
pub mod api_client;
pub use api_client::{ApiClaim, ApiClient, ApiClientError, ApiClientKind, ApiClientResult};
pub mod billing_account;
pub use billing_account::{BillingAccount, BillingAccountError, BillingAccountResult};
pub mod jwt_key;
pub use jwt_key::{
    create_jwt_key_if_missing, get_jwt_signing_key, get_jwt_validation_key, validate_bearer_token,
    validate_bearer_token_api_client, JwtKeyError, JwtKeyResult,
};
pub mod update_clock;
pub use update_clock::{
    init_update_clock_service, next_update_clock, UpdateClock, UpdateClockClient, UpdateClockError,
    UpdateClockResult, UpdateClockService,
};
pub mod event;
pub use event::{Event, EventError, EventKind, EventResult, EventStatus};
pub mod event_log;
pub use event_log::{EventLog, EventLogError, EventLogLevel, EventLogResult};
pub mod output_line;
pub use output_line::{OutputLine, OutputLineStream};
pub mod secret;
pub use secret::{
    EncryptedSecret, Secret, SecretAlgorithm, SecretError, SecretKind, SecretObjectType,
    SecretResult, SecretVersion,
};
pub mod edge;
pub use edge::{Edge, EdgeError, EdgeKind, EdgeResult, Vertex};
pub mod change_set;
pub use change_set::{
    ChangeSet, ChangeSetError, ChangeSetParticipant, ChangeSetResult, ChangeSetStatus, PatchOps,
    PatchReply, PatchRequest,
};
pub mod edit_session;
pub use edit_session::{EditSession, EditSessionError, EditSessionResult};
pub mod si_change_set;
pub use si_change_set::{SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiChangeSetResult};
pub mod system;
pub use system::{System, SystemError, SystemResult};
pub mod entity;
pub use entity::{calculate_properties, Entity, EntityError, EntityResult};
pub mod resource;
pub use resource::{Resource, ResourceError, ResourceHealth, ResourceResult, ResourceStatus};
pub mod node;
pub use node::{Node, NodeError, NodeKind, NodeResult, Position};
pub mod ops;
pub use ops::{
    OpEntityAction, OpEntityDelete, OpEntitySet, OpError, OpReply, OpRequest, OpResult, OpSetName,
    SiOp,
};
pub mod update;
pub use update::{websocket_run, WebsocketToken};

use crate::data::pg::PgTxn;
use crate::PAGE_SECRET_KEY;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("a list model call resulted in no possible query")]
    NoQuery,
    #[error("malformed list item body; missing id")]
    ListItemNoId,
    #[error("invalid query")]
    Query(#[from] QueryError),
    #[error("page token error: {0}")]
    PageToken(#[from] PageTokenError),
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("tokio pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
}
pub type ModelResult<T> = Result<T, ModelError>;

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

// To query something you always need:
//
// * a type of object (billingAccount, workspace, entity, etc.)
// * a tenancy id to search inside (billingAccount, organization)
// * an optional query
//
// We need the count of the number of values, and their values for the prepared statement
// AND we need the query string.

pub async fn list_model(
    txn: &PgTxn<'_>,
    table_name: impl Into<String>,
    tenant_id: impl Into<String>,
    mut query: Option<Query>,
    mut page_size: Option<u32>,
    mut order_by: Option<String>,
    mut order_by_direction: Option<OrderByDirection>,
    page_token: Option<PageToken>,
) -> ModelResult<ListReply> {
    let table_name = table_name.into();
    let tenant_id = tenant_id.into();
    let mut item_id: Option<String> = None;

    if let Some(page_token) = page_token {
        query = page_token.query;
        page_size = Some(page_token.page_size);
        order_by = Some(page_token.order_by);
        order_by_direction = Some(page_token.order_by_direction);
        item_id = Some(page_token.item_id);
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
    };

    let mut base_query = format!(
        "SELECT obj FROM {table_name} WHERE tenant_ids @> ARRAY[$1::TEXT]",
        table_name = table_name
    );
    let mut params = vec![tenant_id];
    if let Some(user_query) = query.clone() {
        let user_query_string = user_query.as_pgsql(&mut params)?;
        base_query.push_str(" AND ");
        base_query.push_str(&user_query_string);
    }
    // make order by break for now - always do by id, and fuck it. It's a SQL injection if we don't
    // validate it.
    let query_string = format!(
        "{base_query} ORDER BY id {order_by_direction}",
        base_query = base_query,
        order_by_direction = order_by_direction.as_ref().unwrap(),
    );

    let params_refs: Vec<_> = params
        .iter()
        .map(|p| p as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();
    dbg!(&query_string);
    dbg!(&params_refs);
    let rows = txn.query(&query_string[..], &params_refs[..]).await?;
    let mut results: Vec<serde_json::Value> = vec![];
    for row in rows.iter() {
        let obj: serde_json::Value = row.try_get("obj")?;
        results.push(obj);
    }
    //trace!(?query_string, "query model");
    //let results: Vec<serde_json::Value> = db.query(query_string, None).await?;
    let total_count = results.len() as u32;
    //trace!(?total_count, ?results, "query model results");

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
                query,
                page_size: page_size.unwrap(),
                order_by: order_by.unwrap(),
                order_by_direction: order_by_direction.unwrap(),
                item_id: next_item_id,
            };
            // Safe because we initialized the constant at the top
            let sealed_token = page_token.seal(unsafe { PAGE_SECRET_KEY.as_ref().unwrap() })?;
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

pub fn generate_name(name: Option<String>) -> String {
    if name.is_some() {
        return name.unwrap();
    }
    let mut name_generator = names::Generator::with_naming(names::Name::Numbered);
    let name = name_generator.next().unwrap();
    return name;
}
