use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{
    ComponentError as DalComponentError, ComponentId, SchemaError, StandardModelError, SystemId,
    WsEventError,
};
use std::convert::Infallible;
use thiserror::Error;

pub mod get_component_metadata;
pub mod get_resource;
pub mod list_components_names_only;
pub mod list_qualifications;
pub mod sync_resource;

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("entity error: {0}")]
    Component(#[from] DalComponentError),
    #[error("not found")]
    NotFound,
    #[error("schema not found")]
    SchemaNotFound,
    #[error("invalid request")]
    InvalidRequest,
    #[error("schema error: {0}")]
    SchemaError(#[from] SchemaError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("resource not found")]
    ResourceNotFound(ComponentId, SystemId),
}

pub type ComponentResult<T> = std::result::Result<T, ComponentError>;

impl IntoResponse for ComponentError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            ComponentError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentError::SchemaNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new()
        .route(
            "/list_components_names_only",
            get(list_components_names_only::list_components_names_only),
        )
        .route(
            "/get_component_metadata",
            get(get_component_metadata::get_component_metadata),
        )
        .route(
            "/list_qualifications",
            get(list_qualifications::list_qualifications),
        )
        .route("/get_resource", get(get_resource::get_resource))
        .route("/sync_resource", post(sync_resource::sync_resource))
}
