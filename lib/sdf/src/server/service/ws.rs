use std::convert::Infallible;

use axum::{
    body::{Bytes, Full},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use thiserror::Error;

pub mod billing_account_updates;

#[derive(Debug, Error)]
pub enum WsError {
    #[error("poop")]
    Poop,
}

pub type WsResult<T> = std::result::Result<T, WsError>;

impl IntoResponse for WsError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() }
        }));

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new().route(
        "/billing_account_updates",
        get(billing_account_updates::billing_account_updates),
    )
}
