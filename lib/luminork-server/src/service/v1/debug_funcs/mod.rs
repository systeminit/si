use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        get,
        post,
    },
};
use dal::ComponentId;
use serde::Deserialize;
use si_id::DebugFuncJobStateId;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod exec_debug_func;
pub mod get_debug_func_state;

pub use exec_debug_func::{
    ExecDebugFuncV1Request,
    ExecDebugFuncV1Response,
};
pub use get_debug_func_state::GetDebugFuncJobStateV1Response;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DebugFuncsError {
    #[error("component not found with id: {0}")]
    ComponentNotFound(ComponentId),
    #[error("debug function job state not found: {0}")]
    DebugFuncsJobStateNotFound(DebugFuncJobStateId),
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type DebugFuncsResult<T> = Result<T, DebugFuncsError>;

impl IntoResponse for DebugFuncsError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

#[derive(Deserialize, ToSchema)]
pub struct DebugFuncsV1RequestPath {
    #[schema(value_type = String)]
    pub debug_func_job_state_id: DebugFuncJobStateId,
}

impl From<JsonRejection> for DebugFuncsError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                Self::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                Self::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => Self::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => Self::Validation(format!("JSON validation error: {rejection}")),
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for DebugFuncsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            Self::DebugFuncsJobStateNotFound(_) | Self::ComponentNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Self::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(exec_debug_func::exec_debug_func))
        .nest(
            "/:debug_func_job_state_id",
            Router::new().route("/", get(get_debug_func_state::get_debug_func_state)),
        )
}
