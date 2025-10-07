//! Common types and utilities for the v1 API

use axum::{
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use thiserror::Error;
use utoipa::ToSchema;

/// Standard success response format for v1 API
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiSuccess<T> {
    pub data: T,
}

/// Standard error response format for v1 API
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "message": "Invalid request data",
    "statusCode": 422,
    "code": 4001
}))]
pub struct ApiError {
    #[schema(example = "Invalid request data")]
    pub message: String,
    #[schema(example = 422)]
    pub status_code: u16,
    #[schema(example = 4001)]
    pub code: Option<i32>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(status: StatusCode, message: String) -> Self {
        Self {
            message,
            status_code: status.as_u16(),
            code: None,
        }
    }

    /// Create a new API error with a code
    pub fn with_code(status: StatusCode, message: String, code: i32) -> Self {
        Self {
            message,
            status_code: status.as_u16(),
            code: Some(code),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Store message before moving self
        let message = self.message.clone();

        if status.is_client_error() {
            warn!(err=?self, si.error.message = %message );
        } else if status.is_server_error() {
            error!(err=?self, si.error.message = %message );
        }
        let body = axum::Json(self);

        (status, body).into_response()
    }
}

/// Common error types that can be reused across v1 API modules
#[remain::sorted]
#[derive(Debug, Error)]
pub enum CommonError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("transaction error: {0}")]
    Transaction(#[from] dal::TransactionsError),
    #[error("workspace error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

/// Helper trait for converting errors to responses
pub trait ErrorIntoResponse: std::error::Error {
    /// Convert an error to a status code and message
    fn status_and_message(&self) -> (StatusCode, String);

    /// Convert an error to an API response
    fn to_api_response(&self) -> Response {
        let (status, message) = self.status_and_message();
        ApiError::new(status, message).into_response()
    }
}

/// Common path parameters used in multiple endpoints
#[derive(Deserialize, ToSchema)]
pub struct WorkspaceIdParam {
    #[schema(value_type = String)]
    pub workspace_id: dal::WorkspacePk,
}

/// Common path parameters used in multiple endpoints
#[derive(Deserialize, ToSchema)]
pub struct ChangeSetIdParam {
    #[schema(value_type = String)]
    pub change_set_id: dal::ChangeSetId,
}

/// Common path parameters used in multiple endpoints
#[derive(Deserialize, ToSchema)]
pub struct ComponentIdParam {
    #[schema(value_type = String)]
    pub component_id: dal::ComponentId,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    #[schema(example = "50", nullable = true, value_type = Option<u32>)]
    pub limit: Option<u32>,
    #[schema(example = "01H9ZQD35JPMBGHH69BT0Q79VY", nullable = true, value_type = Option<String>)]
    pub cursor: Option<String>,
    #[schema(value_type = Option<bool>)]
    pub include_codegen: Option<bool>,
}
