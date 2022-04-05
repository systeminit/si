use std::convert::Infallible;

use axum::{
    body::{Bytes, Full},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dal::{
    ComponentError as DalComponentError, ComponentId, ReadTenancyError, SchemaError,
    StandardModelError, SystemId, TransactionsError, WsEventError,
};

use thiserror::Error;

pub mod get_code;
pub mod get_component_metadata;
pub mod get_resource;
pub mod list_components_with_schema_and_variant;
pub mod list_qualifications;
pub mod sync_resource;

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("entity error: {0}")]
    Component(#[from] DalComponentError),
    #[error("component name not found")]
    ComponentNameNotFound,
    #[error("component not found")]
    ComponentNotFound,
    #[error("invalid request")]
    InvalidRequest,
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("resource not found")]
    ResourceNotFound(ComponentId, SystemId),
    #[error("schema error: {0}")]
    SchemaError(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
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
            "/list_components_with_schema_and_variant",
            get(list_components_with_schema_and_variant::list_components_with_schema_and_variant),
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
        .route("/get_code", get(get_code::get_code))
}
