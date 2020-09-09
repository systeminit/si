use serde::Serialize;
use si_data::Db;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{reject::Reject, Filter, Rejection, Reply};

use tracing::error;

use crate::models;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("database error: {0}")]
    Database(#[from] si_data::DataError),
    #[error("account error: {0}")]
    Account(#[from] si_account::error::AccountError),
    #[error("invalid json pointer: {0}")]
    InvalidJsonPointer(String),
    #[error("invalid json value: {0}")]
    InvalidJsonValue(#[from] serde_json::Error),
    #[error("mismatched json value: {0}")]
    MismatchedJsonValue(String),
}

impl Reject for HandlerError {}
impl From<HandlerError> for warp::reject::Rejection {
    fn from(e: HandlerError) -> Self {
        warp::reject::custom(e)
    }
}

pub async fn update_entity_field(
    entity_id: String,
    db: Db,
    user_id: String,
    billing_account_id: String,
    set_field: models::SetField,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, user_id, billing_account_id).await?;
    let mut object: serde_json::Value = db.get(&entity_id).await.map_err(HandlerError::from)?;
    let object_field = object
        .pointer_mut(&set_field.pointer)
        .ok_or(HandlerError::InvalidJsonPointer(set_field.pointer.clone()))?;
    let new_value: serde_json::Value =
        serde_json::from_str(&set_field.value).map_err(HandlerError::from)?;
    if (new_value.is_string() && object_field.is_string())
        || (new_value.is_array() && object_field.is_array())
        || (new_value.is_object() && object_field.is_object())
        || (new_value.is_number() && object_field.is_number())
        || (new_value.is_boolean() && object_field.is_boolean())
        || (object_field.is_null())
    {
        *object_field = new_value;
    } else {
        return Err(HandlerError::MismatchedJsonValue(set_field.value))
            .map_err(HandlerError::from)?;
    }
    db.update_raw(&object).await.map_err(HandlerError::from)?;
    Ok(warp::reply::json(&object))
}

pub async fn get_entity(
    entity_id: String,
    db: Db,
    user_id: String,
    billing_account_id: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, user_id, billing_account_id).await?;
    let result: serde_json::Value = db.get(entity_id).await.map_err(HandlerError::from)?;
    Ok(warp::reply::json(&result))
}

pub async fn list_entities(
    db: Db,
    user_id: String,
    billing_account_id: String,
    query: models::ListQuery,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, user_id, billing_account_id).await?;
    let list_result = match query.page_token {
        Some(token) => db
            .list_by_page_token_raw_by_type(token, query.type_name.unwrap_or("".to_string()))
            .await
            .map_err(HandlerError::from)?,
        None => {
            let contained_within = match query.scope_by_tenant_id {
                Some(contained_within) => contained_within,
                None => {
                    return Err(HandlerError::from(
                        si_data::DataError::MissingScopeByTenantId,
                    ))?
                }
            };

            db.list_raw_by_type(
                &query.query,
                query.page_size.unwrap_or(10),
                query.order_by.unwrap_or("".to_string()),
                query.order_by_direction.map(|ob| ob as i32).unwrap_or(0),
                &contained_within,
                "",
                query.type_name.unwrap_or("".to_string()),
            )
            .await
            .map_err(HandlerError::from)?
        }
    };

    let response = models::ListResponse {
        items: list_result.items,
        total_count: list_result.total_count,
        next_item_id: list_result.next_item_id,
        page_token: list_result.page_token,
    };
    Ok(warp::reply::json(&response))
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn authorize(
    db: &Db,
    user_id: String,
    billing_account_id: String,
) -> Result<si_account::authorize::Authentication, HandlerError> {
    let auth = si_account::authorize::Authentication::new(user_id, billing_account_id);
    auth.authorize_on_billing_account(db, "entity").await?;
    Ok(auth)
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
    } else if let Some(HandlerError::Account(err)) = err.find() {
        code = StatusCode::UNAUTHORIZED;
        tracing::debug!("request unauthorized: {}", err);
        message = err.to_string();
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
        message = "UNHANDLED_REJECTION".to_string();
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
