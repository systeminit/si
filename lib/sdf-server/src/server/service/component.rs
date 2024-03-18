use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::attribute::value::AttributeValueError;
use dal::component::ComponentId;
use dal::property_editor::PropertyEditorError;
use dal::validation::resolver::ValidationResolverError;
use dal::{ActionPrototypeError, ComponentError as DalComponentError, StandardModelError};
use dal::{ChangeSetPointerError, TransactionsError};
use thiserror::Error;

use crate::server::state::AppState;

pub mod get_property_editor_schema;
pub mod get_property_editor_validations;
pub mod get_property_editor_values;
pub mod update_property_editor_value;

// pub mod debug;
pub mod delete_property_editor_value;
pub mod get_diff;
// pub mod get_resource;
pub mod insert_property_editor_value;
// pub mod json;
pub mod get_actions;
pub mod list_qualifications;
// pub mod list_resources;
// pub mod refresh;
// pub mod resource_domain_diff;
pub mod debug;
pub mod get_code;
pub mod set_type;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("action prototype: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    // #[error("attribute context builder error: {0}")]
    // AttributeContextBuilder(#[from] AttributeContextBuilderError),
    // #[error("attribute prototype error: {0}")]
    // AttributePrototype(#[from] AttributePrototypeError),
    // #[error("attribute prototype argument error: {0}")]
    // AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    // #[error("attribute prototype not found")]
    // AttributePrototypeNotFound,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    // #[error("attribute value not found")]
    // AttributeValueNotFound,
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    // #[error("change status error: {0}")]
    // ChangeStatus(#[from] ChangeStatusError),
    // #[error("component debug view error: {0}")]
    // ComponentDebug(String),
    // #[error("component debug view error: {0}")]
    // ComponentDebugView(#[from] ComponentDebugViewError),
    // #[error("component name not found")]
    // ComponentNameNotFound,
    // #[error("component view error: {0}")]
    // ComponentView(#[from] ComponentViewError),
    #[error("dal component error: {0}")]
    DalComponent(#[from] DalComponentError),
    // #[error("dal schema error: {0}")]
    // DalSchema(#[from] DalSchemaError),
    // #[error("diagram error: {0}")]
    // Diagram(#[from] DiagramError),
    // #[error("func error: {0}")]
    // Func(#[from] FuncError),
    // #[error("func binding error: {0}")]
    // FuncBinding(#[from] FuncBindingError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    // #[error("identity func not found")]
    // IdentityFuncNotFound,
    // #[error("invalid request")]
    // InvalidRequest,
    #[error("invalid visibility")]
    InvalidVisibility,
    // #[error("property value key not found")]
    // KeyNotFound,
    // #[error(transparent)]
    // Nats(#[from] si_data_nats::NatsError),
    // #[error("node error: {0}")]
    // Node(#[from] NodeError),
    #[error("component not found for id: {0}")]
    NotFound(ComponentId),
    // #[error(transparent)]
    // Pg(#[from] si_data_pg::PgError),
    // #[error(transparent)]
    // Prop(#[from] PropError),
    #[error("property editor error: {0}")]
    PropertyEditor(#[from] PropertyEditorError),
    // #[error("prop not found for id: {0}")]
    // PropNotFound(PropId),
    // #[error("reconciliation prototype: {0}")]
    // ReconciliationPrototype(#[from] ReconciliationPrototypeError),
    // #[error("can't delete attribute value for root prop")]
    // RootPropAttributeValue,
    // #[error("schema error: {0}")]
    // Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    // #[error("system id is required: ident_nil_v1() was provided")]
    // SystemIdRequired,
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    // #[error("ws event error: {0}")]
    // WsEvent(#[from] WsEventError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationResolverError),
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
        //.route(
        //            "/get_property_editor_validations",
        //            get(get_property_editor_validations::get_property_editor_validations),
        //        )
        .route(
            "/list_qualifications",
            get(list_qualifications::list_qualifications),
        )
        .route("/get_code", get(get_code::get_code))
        .route("/get_diff", get(get_diff::get_diff))
        .route(
            "/update_property_editor_value",
            post(update_property_editor_value::update_property_editor_value),
        )
        .route(
            "/insert_property_editor_value",
            post(insert_property_editor_value::insert_property_editor_value),
        )
        .route(
            "/delete_property_editor_value",
            post(delete_property_editor_value::delete_property_editor_value),
        )
        .route("/set_type", post(set_type::set_type))
        // .route("/refresh", post(refresh::refresh))
        // .route("/resource_domain_diff", get(resource_domain_diff::get_diff))
        .route("/debug", get(debug::debug_component))
    // .route("/json", get(json::json))
}
