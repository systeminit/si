use axum::{
    http::StatusCode, response::IntoResponse, response::Response, routing::get, Json, Router,
};
use dal::{ComponentError, ComponentId, StandardModelError, TransactionsError};
use thiserror::Error;

pub mod list_resources_by_component;

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("component name not found: {0}")]
    ComponentNameNotFound(ComponentId),
}

pub type ResourceResult<T> = std::result::Result<T, ResourceError>;

impl IntoResponse for ResourceError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new().route(
        "/list_resources_by_component",
        get(list_resources_by_component::list_resources_by_component),
    )
}
