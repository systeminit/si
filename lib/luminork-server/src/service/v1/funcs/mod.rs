use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        get,
        post,
        put,
    },
};
use dal::{
    FuncId,
    SchemaVariantError,
    func::authoring::FuncAuthoringError,
};
use serde::Deserialize;
use si_id::FuncRunId;
use si_layer_cache::LayerDbError;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod get_func;
pub mod get_func_run;
pub mod unlock_func;
pub mod update_func;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncsError {
    #[error("funcs error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func not found: {0}")]
    FuncNotFound(FuncId),
    #[error("func run not found: {0}")]
    FuncRunNotFound(FuncRunId),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("trying to modify locked func: {0}")]
    LockedFunc(FuncId),
    #[error("changes not permitted on HEAD change set")]
    NotPermittedOnHead,
    #[error("layer db error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("funcs can only be unlocked for unlocked schema variants")]
    SchemaVariantMustBeUnlocked,
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type FuncsResult<T> = Result<T, FuncsError>;

#[derive(Deserialize, ToSchema)]
pub struct FuncRunV1RequestPath {
    #[schema(value_type = String)]
    pub func_run_id: FuncRunId,
}

#[derive(Deserialize, ToSchema)]
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
                FuncsError::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                FuncsError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => FuncsError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => FuncsError::Validation(format!("JSON validation error: {rejection}")),
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for FuncsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            FuncsError::FuncRunNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            FuncsError::FuncNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            FuncsError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            FuncsError::SchemaVariantMustBeUnlocked => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest(
            "/:func_id",
            Router::new()
                .route("/", get(get_func::get_func))
                .route("/unlock", post(unlock_func::unlock_func))
                .route("/", put(update_func::update_func)),
        )
        .nest(
            "/runs",
            Router::new().route("/:func_run_id", get(get_func_run::get_func_run)),
        )
}
