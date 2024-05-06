use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::prop::PropError;
use dal::property_editor::PropertyEditorError;
use dal::validation::ValidationError;
use dal::{
    attribute::value::debug::AttributeDebugViewError, component::ComponentId, PropId,
    SecretError as DalSecretError, WsEventError,
};
use dal::{attribute::value::AttributeValueError, component::debug::ComponentDebugViewError};
use dal::{ChangeSetError, TransactionsError};
use dal::{
    ComponentError as DalComponentError, DeprecatedActionPrototypeError, StandardModelError,
};
use thiserror::Error;

use crate::server::state::AppState;

pub mod delete_property_editor_value;
pub mod get_actions;
pub mod get_diff;
pub mod get_property_editor_schema;
pub mod get_property_editor_values;
pub mod get_resource;
pub mod insert_property_editor_value;
pub mod json;
pub mod list_qualifications;
pub mod update_property_editor_value;
// pub mod list_resources;
pub mod refresh;
// pub mod resource_domain_diff;
pub mod debug;
pub mod get_code;
pub mod restore_default_function;
pub mod set_type;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("action prototype: {0}")]
    ActionPrototype(#[from] DeprecatedActionPrototypeError),
    #[error("attribute debug view error: {0}")]
    AttributeDebugViewError(#[from] AttributeDebugViewError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component debug view error: {0}")]
    ComponentDebugView(#[from] ComponentDebugViewError),
    #[error("dal component error: {0}")]
    DalComponent(#[from] DalComponentError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("invalid visibility")]
    InvalidVisibility,
    #[error("component not found for id: {0}")]
    NotFound(ComponentId),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error("property editor error: {0}")]
    PropertyEditor(#[from] PropertyEditorError),
    #[error("prop not found for id: {0}")]
    PropNotFound(PropId),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("dal secret error: {0}")]
    Secret(#[from] DalSecretError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/get_actions", get(get_actions::get_actions))
        .route(
            "/get_property_editor_schema",
            get(get_property_editor_schema::get_property_editor_schema),
        )
        .route(
            "/get_property_editor_values",
            get(get_property_editor_values::get_property_editor_values),
        )
        .route(
            "/list_qualifications",
            get(list_qualifications::list_qualifications),
        )
        .route("/get_code", get(get_code::get_code))
        .route("/get_diff", get(get_diff::get_diff))
        .route("/get_resource", get(get_resource::get_resource))
        .route(
            "/upsert_property_editor_value",
            post(update_property_editor_value::upsert_property_editor_value),
        )
        .route(
            "/delete_property_editor_value",
            post(delete_property_editor_value::delete_property_editor_value),
        )
        .route(
            "/restore_default_function",
            post(restore_default_function::restore_default_function),
        )
        .route("/set_type", post(set_type::set_type))
        .route("/refresh", post(refresh::refresh))
        // .route("/resource_domain_diff", get(resource_domain_diff::get_diff))
        .route("/debug", get(debug::debug_component))
        .route("/json", get(json::json))
}
