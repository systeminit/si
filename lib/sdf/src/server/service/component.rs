use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    node::NodeError, property_editor::PropertyEditorError, AttributeValueError,
    ComponentError as DalComponentError, ComponentId, DiagramError, ReadTenancyError,
    SchemaError as DalSchemaError, StandardModelError, SystemId, TransactionsError, WsEventError,
};
use thiserror::Error;

use crate::service::schema::SchemaError;

pub mod check_qualifications;
pub mod generate_code;
pub mod get_code;
pub mod get_components_metadata;
pub mod get_diff;
pub mod get_property_editor_schema;
pub mod get_property_editor_validations;
pub mod get_property_editor_values;
pub mod get_resource;
pub mod insert_property_editor_value;
pub mod list_components_identification;
pub mod list_qualifications;
pub mod sync_resource;
pub mod update_property_editor_value;

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("entity error: {0}")]
    Component(#[from] DalComponentError),
    #[error("component name not found")]
    ComponentNameNotFound,
    #[error("component not found")]
    ComponentNotFound,
    #[error("dal schema error: {0}")]
    DalSchema(#[from] DalSchemaError),
    #[error("invalid request")]
    InvalidRequest,
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("resource not found")]
    ResourceNotFound(ComponentId, SystemId),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),

    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("system id is required: -1 was provided")]
    SystemIdRequired,
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("invalid visibility")]
    InvalidVisibility,
    #[error("property editor error: {0}")]
    PropertyEditor(#[from] PropertyEditorError),
}

pub type ComponentResult<T> = std::result::Result<T, ComponentError>;

impl IntoResponse for ComponentError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ComponentError::SchemaNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentError::InvalidVisibility => (StatusCode::NOT_FOUND, self.to_string()),
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
            "/list_components_identification",
            get(list_components_identification::list_components_identification),
        )
        .route(
            "/get_components_metadata",
            get(get_components_metadata::get_components_metadata),
        )
        .route(
            "/list_qualifications",
            get(list_qualifications::list_qualifications),
        )
        .route("/get_resource", get(get_resource::get_resource))
        .route("/sync_resource", post(sync_resource::sync_resource))
        .route(
            "/check_qualifications",
            post(check_qualifications::check_qualifications),
        )
        .route("/get_code", get(get_code::get_code))
        .route("/get_diff", get(get_diff::get_diff))
        .route("/generate_code", post(generate_code::generate_code))
        .route(
            "/get_property_editor_schema",
            get(get_property_editor_schema::get_property_editor_schema),
        )
        .route(
            "/get_property_editor_values",
            get(get_property_editor_values::get_property_editor_values),
        )
        .route(
            "/update_property_editor_value",
            post(update_property_editor_value::update_property_editor_value),
        )
        .route(
            "/insert_property_editor_value",
            post(insert_property_editor_value::insert_property_editor_value),
        )
        .route(
            "/get_property_editor_validations",
            get(get_property_editor_validations::get_property_editor_validations),
        )
}
