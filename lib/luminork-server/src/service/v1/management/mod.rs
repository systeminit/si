use axum::{http::StatusCode, response::IntoResponse, routing::post, Router};
use sdf_core::api_error::ApiError;
use thiserror::Error;

use crate::AppState;

pub mod run_prototype;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementApiError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] dal::func::authoring::FuncAuthoringError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("management error: {0}")]
    Management(#[from] dal::management::ManagementError),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] dal::management::prototype::ManagementPrototypeError),
    #[error("management prototype execution failure: {0}")]
    ManagementPrototypeExecutionFailure(dal::management::prototype::ManagementPrototypeId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] dal::schema::variant::authoring::VariantAuthoringError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

impl crate::service::v1::common::ErrorIntoResponse for ManagementApiError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ManagementApiError::ManagementPrototype(
                dal::management::prototype::ManagementPrototypeError::FuncExecutionFailure(message),
            ) => (StatusCode::BAD_REQUEST, message.clone()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

impl IntoResponse for ManagementApiError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        let (status, message) = self.status_and_message();
        ApiError::new(status, message).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/prototype/:management_prototype_id",
        Router::new().route(
            "/:component_id/:view_id",
            post(run_prototype::run_prototype),
        ),
    )
}
