use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{StatusUpdateError, TransactionsError};
use hyper::StatusCode;
use thiserror::Error;

use crate::server::state::AppState;

pub mod list_active_statuses;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum StatusError {
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    StatusUpdate(#[from] StatusUpdateError),
}

pub type StatusResult<T> = std::result::Result<T, StatusError>;

impl IntoResponse for StatusError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/list-active-statuses",
        get(list_active_statuses::list_active_statuses),
    )
}
