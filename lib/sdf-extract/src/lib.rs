use axum::{http::StatusCode, Json};
use std::fmt;

pub mod change_set;
pub mod request;
pub mod services;
pub mod v1;
pub mod workspace;

pub use services::*;

pub type ErrorResponse = (StatusCode, Json<serde_json::Value>);

pub fn internal_error(message: impl fmt::Display) -> ErrorResponse {
    let status_code = StatusCode::INTERNAL_SERVER_ERROR;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

pub fn bad_request(message: impl fmt::Display) -> ErrorResponse {
    let status_code = StatusCode::BAD_REQUEST;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

pub fn unauthorized_error(message: impl fmt::Display) -> ErrorResponse {
    let status_code = StatusCode::UNAUTHORIZED;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

pub fn not_found_error(message: &str) -> ErrorResponse {
    let status_code = StatusCode::NOT_FOUND;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message,
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}
