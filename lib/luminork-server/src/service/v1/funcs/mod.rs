use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use dal::FuncId;
use serde::Deserialize;
use si_id::FuncRunId;
use si_layer_cache::LayerDbError;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod get_func;
pub mod get_func_run;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncsError {
    #[error("funcs error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("func not found: {0}")]
    FuncNotFound(FuncId),
    #[error("func run not found: {0}")]
    FuncRunNotFound(FuncRunId),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type FuncsResult<T> = Result<T, FuncsError>;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FuncRunV1RequestPath {
    #[schema(value_type = String)]
    pub func_run_id: FuncRunId,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FuncV1RequestPath {
    #[schema(value_type = String)]
    pub func_id: FuncId,
}

impl IntoResponse for FuncsError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl From<JsonRejection> for FuncsError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                FuncsError::Validation(format!("Invalid JSON data format: {}", rejection))
            }
            JsonRejection::JsonSyntaxError(_) => {
                FuncsError::Validation(format!("Invalid JSON syntax: {}", rejection))
            }
            JsonRejection::MissingJsonContentType(_) => FuncsError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => FuncsError::Validation(format!("JSON validation error: {}", rejection)),
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for FuncsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            FuncsError::FuncRunNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            FuncsError::FuncNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            FuncsError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest(
            "/:func_id",
            Router::new().route("/", get(get_func::get_func)),
        )
        .nest(
            "/runs",
            Router::new().route("/:func_run_id", get(get_func_run::get_func_run)),
        )
}
