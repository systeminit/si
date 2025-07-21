use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use si_id::ManagementFuncJobStateId;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod get_management_func_run_state;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementFuncsError {
    #[error("management func job state not found: {0}")]
    ManagementFuncJobStateNotFound(ManagementFuncJobStateId),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type ManagementFuncsResult<T> = Result<T, ManagementFuncsError>;

impl IntoResponse for ManagementFuncsError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ManagementFuncJobStateV1RequestPath {
    #[schema(value_type = String)]
    pub management_func_job_state_id: ManagementFuncJobStateId,
}

impl From<JsonRejection> for ManagementFuncsError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                ManagementFuncsError::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                ManagementFuncsError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => ManagementFuncsError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => ManagementFuncsError::Validation(format!("JSON validation error: {rejection}")),
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for ManagementFuncsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ManagementFuncsError::ManagementFuncJobStateNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/:management_func_job_state_id",
        Router::new().route(
            "/",
            get(get_management_func_run_state::get_management_func_run_state),
        ),
    )
}
