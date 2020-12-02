use crate::data::Db;
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use warp::http::StatusCode;
use warp::{reject::Reject, Rejection, Reply};

use std::collections::HashMap;
use std::convert::Infallible;

use crate::models::{
    ApiClaim, ApiClientError, BillingAccountError, ChangeSetError, EdgeError, EditSessionError,
    EntityError, EventError, JwtKeyError, KeyPairError, ModelError, NodeError, OpError, PageToken,
    PageTokenError, Query, QueryError, SecretError, SiStorableError, UserError,
};

pub mod api_clients;
pub mod billing_accounts;
pub mod change_sets;
pub mod cli;
pub mod edges;
pub mod edit_sessions;
pub mod entities;
pub mod nodes;
pub mod secrets;
pub mod updates;
pub mod users;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("database error: {0}")]
    Database(#[from] crate::data::DataError),
    #[error("invalid json pointer: {0}")]
    InvalidJsonPointer(String),
    #[error("invalid json value: {0}")]
    InvalidJsonValue(#[from] serde_json::Error),
    #[error("mismatched json value: {0}")]
    MismatchedJsonValue(String),
    #[error("error in storable: {0}")]
    Storable(#[from] SiStorableError),
    #[error("error in the model layer: {0}")]
    Model(#[from] ModelError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edit session error: {0}")]
    EditSession(#[from] EditSessionError),
    #[error("op error: {0}")]
    OpError(#[from] OpError),
    #[error("billing account error: {0}")]
    BillingAccount(#[from] BillingAccountError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("call is unauthorized")]
    Unauthorized,
    #[error("jwt error fetching signing key: {0}")]
    JwtKey(#[from] JwtKeyError),
    #[error("error signing jwt claim: {0}")]
    JwtClaim(String),
    #[error("query error: {0}")]
    Query(#[from] QueryError),
    #[error("page token error: {0}")]
    PageToken(#[from] PageTokenError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("event error: {0}")]
    Event(#[from] EventError),
    #[error("api client error: {0}")]
    ApiClient(#[from] ApiClientError),
    #[error("invalid request")]
    InvalidRequest,
}

pub type HandlerResult<T> = Result<T, HandlerError>;

impl Reject for HandlerError {}
impl From<HandlerError> for warp::reject::Rejection {
    fn from(err: HandlerError) -> Self {
        match err {
            HandlerError::Model(ref inner) => match inner {
                ModelError::GetNotFound
                | ModelError::Couchbase(couchbase::error::CouchbaseError::KeyDoesNotExist) => {
                    warp::reject::not_found()
                }
                _ => warp::reject::custom(err),
            },
            _ => warp::reject::custom(err),
        }
    }
}

pub async fn list_models(
    db: Db,
    token: String,
    type_name: String,
    request: crate::models::ListRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        &type_name,
        "list",
    )
    .await?;

    let query = if let Some(query) = request.query {
        Some(Query::from_url_string(query).map_err(HandlerError::from)?)
    } else {
        None
    };

    let page_token = if let Some(page_token) = request.page_token {
        Some(PageToken::unseal(&page_token, &db.page_secret_key).map_err(HandlerError::from)?)
    } else {
        None
    };

    let reply = crate::models::list_model(
        &db,
        query,
        request.page_size,
        request.order_by,
        request.order_by_direction,
        page_token,
        Some(type_name),
        Some(claim.billing_account_id.clone()),
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&reply))
}

pub async fn get_model_change_set(
    id: String,
    db: Db,
    token: String,
    type_name: String,
    request: crate::models::GetRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        &type_name,
        "get",
    )
    .await?;

    let item = crate::models::get_model_change_set(
        &db,
        id,
        type_name,
        claim.billing_account_id,
        request.change_set_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = crate::models::GetReply { item };
    Ok(warp::reply::json(&reply))
}

//pub async fn update_entity_field(
//    entity_id: String,
//    db: Db,
//    user_id: String,
//    billing_account_id: String,
//    set_field: models::SetField,
//) -> Result<impl warp::Reply, warp::reject::Rejection> {
//    authorize(&db, user_id, billing_account_id).await?;
//    let mut object: serde_json::Value = db.get(&entity_id).await.map_err(HandlerError::from)?;
//    let object_field = object
//        .pointer_mut(&set_field.pointer)
//        .ok_or(HandlerError::InvalidJsonPointer(set_field.pointer.clone()))?;
//    let new_value: serde_json::Value =
//        serde_json::from_str(&set_field.value).map_err(HandlerError::from)?;
//    if (new_value.is_string() && object_field.is_string())
//        || (new_value.is_array() && object_field.is_array())
//        || (new_value.is_object() && object_field.is_object())
//        || (new_value.is_number() && object_field.is_number())
//        || (new_value.is_boolean() && object_field.is_boolean())
//        || (object_field.is_null())
//    {
//        *object_field = new_value;
//    } else {
//        return Err(HandlerError::MismatchedJsonValue(set_field.value))
//            .map_err(HandlerError::from)?;
//    }
//    //db.update_raw(&object).await.map_err(HandlerError::from)?;
//    Ok(warp::reply::json(&object))
//}
//
//pub async fn get_entity(
//    entity_id: String,
//    db: Db,
//    user_id: String,
//    billing_account_id: String,
//) -> Result<impl warp::Reply, warp::reject::Rejection> {
//    authorize(&db, user_id, billing_account_id).await?;
//    let result: serde_json::Value = db.get(entity_id).await.map_err(HandlerError::from)?;
//    Ok(warp::reply::json(&result))
//}
//
//pub async fn list_entities(
//    db: Db,
//    user_id: String,
//    billing_account_id: String,
//    query: models::ListQuery,
//) -> Result<impl warp::Reply, warp::reject::Rejection> {
//    authorize(&db, user_id, billing_account_id).await?;
//    let list_result = match query.page_token {
//        Some(token) => db
//            .list_by_page_token_raw_by_type(token, query.type_name.unwrap_or("".to_string()))
//            .await
//            .map_err(HandlerError::from)?,
//        None => {
//            let contained_within = match query.scope_by_tenant_id {
//                Some(contained_within) => contained_within,
//                None => {
//                    return Err(HandlerError::from(
//                        crate::data::DataError::MissingScopeByTenantId,
//                    ))?
//                }
//            };
//
//            db.list_raw_by_type(
//                &query.query,
//                query.page_size.unwrap_or(10),
//                query.order_by.unwrap_or("".to_string()),
//                query.order_by_direction.map(|ob| ob as i32).unwrap_or(0),
//                &contained_within,
//                "",
//                query.type_name.unwrap_or("".to_string()),
//            )
//            .await
//            .map_err(HandlerError::from)?
//        }
//    };
//
//    let response = models::ListResponse {
//        items: list_result.items,
//        total_count: list_result.total_count,
//        next_item_id: list_result.next_item_id,
//        page_token: list_result.page_token,
//    };
//    Ok(warp::reply::json(&response))
//}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

pub async fn authenticate(db: &Db, token: impl AsRef<str>) -> HandlerResult<users::SiClaims> {
    let token = token.as_ref();
    let claims = crate::models::jwt_key::JwtKeyPublic::validate_bearer_token(&db, token).await?;
    Ok(claims.custom)
}

pub async fn authorize(
    db: &Db,
    user_id: impl Into<String>,
    billing_account_id: impl Into<String>,
    subject: impl AsRef<str>,
    action: impl AsRef<str>,
) -> HandlerResult<()> {
    let user_id = user_id.into();
    let billing_account_id = billing_account_id.into();
    let subject = subject.as_ref();
    let action = action.as_ref();

    let query = format!(
        "SELECT a.*
           FROM `{bucket}` AS a
           WHERE (
                (a.siStorable.typeName = \"user\" AND a.id = $user_id)
                OR
                (a.siStorable.typeName = \"group\" AND ARRAY_CONTAINS(a.userIds, $user_id)))
             AND ARRAY_CONTAINS(a.siStorable.tenantIds, $tenant_id)
             AND (ARRAY_CONTAINS(a.capabilities, $capability) OR ARRAY_CONTAINS(a.capabilities, $any))",
         bucket = db.bucket_name,
     );
    let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
    named_params.insert("user_id".into(), serde_json::json![&user_id]);
    named_params.insert("tenant_id".into(), serde_json::json![&billing_account_id]);
    named_params.insert(
        "capability".into(),
        serde_json::json![{ "subject": subject, "action": action }],
    );
    named_params.insert(
        "any".into(),
        serde_json::json![{ "subject": "any", "action": "any" }],
    );
    let results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
    if results.len() > 0 {
        Ok(())
    } else {
        Err(HandlerError::Unauthorized)
    }
}

pub async fn authenticate_api_client(db: &Db, token: impl AsRef<str>) -> HandlerResult<ApiClaim> {
    let token = token.as_ref();
    let claims =
        crate::models::jwt_key::JwtKeyPublic::validate_bearer_token_api_client(&db, token).await?;
    Ok(claims.custom)
}

pub async fn authorize_api_client(
    db: &Db,
    api_client_id: impl Into<String>,
    billing_account_id: impl Into<String>,
    subject: impl AsRef<str>,
    action: impl AsRef<str>,
) -> HandlerResult<()> {
    let api_client_id = api_client_id.into();
    let billing_account_id = billing_account_id.into();
    let subject = subject.as_ref();
    let action = action.as_ref();

    let query = format!(
        "SELECT a.*
           FROM `{bucket}` AS a
           WHERE (a.siStorable.typeName = \"group\" AND ARRAY_CONTAINS(a.apiClientIds, $api_client_id))
             AND ARRAY_CONTAINS(a.siStorable.tenantIds, $tenant_id)
             AND (ARRAY_CONTAINS(a.capabilities, $capability) OR ARRAY_CONTAINS(a.capabilities, $any))",
         bucket = db.bucket_name,
     );
    let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
    named_params.insert("api_client_id".into(), serde_json::json![&api_client_id]);
    named_params.insert("tenant_id".into(), serde_json::json![&billing_account_id]);
    named_params.insert(
        "capability".into(),
        serde_json::json![{ "subject": subject, "action": action }],
    );
    named_params.insert(
        "any".into(),
        serde_json::json![{ "subject": "any", "action": "any" }],
    );
    let results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
    if results.len() > 0 {
        Ok(())
    } else {
        Err(HandlerError::Unauthorized)
    }
}

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code: StatusCode;
    let message: String;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(HandlerError::Database(err)) = err.find() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = err.to_string();
    } else if let Some(HandlerError::Unauthorized) = err.find() {
        code = StatusCode::UNAUTHORIZED;
        message = String::from("request is unauthorized");
    } else if let Some(header) = err.find::<warp::reject::MissingHeader>() {
        code = StatusCode::UNAUTHORIZED;
        message = format!("{}", header);
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_string();
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("UNHANDLED_REJECTION: {:?}", err);
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
