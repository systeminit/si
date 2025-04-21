use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use dal::ComponentId;
use serde::Deserialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod create_component;
pub mod delete_component;
pub mod get_component;
pub mod update_component;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentsError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("component not found: {0}")]
    ComponentNotFound(String),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("diagram error: {0}")]
    Diagram(#[from] dal::diagram::DiagramError),
    #[error("ambiguous component name reference: {0} (found multiple components with this name)")]
    DuplicateComponentName(String),
    #[error("input socket error: {0}")]
    InputSocket(#[from] dal::socket::input::InputSocketError),
    #[error("prop error: {0}")]
    ManagementPrototype(#[from] dal::management::prototype::ManagementPrototypeError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] dal::socket::output::OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] dal::prop::PropError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

#[derive(Deserialize, ToSchema)]
pub struct ComponentV1RequestPath {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
}

impl IntoResponse for ComponentsError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl crate::service::v1::common::ErrorIntoResponse for ComponentsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ComponentsError::Component(dal::ComponentError::NotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ComponentsError::ComponentNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentsError::DuplicateComponentName(_) => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_component::create_component))
        .nest(
            "/:component_id",
            Router::new()
                .route("/", get(get_component::get_component))
                .route("/", put(update_component::update_component))
                .route("/", delete(delete_component::delete_component)),
        )
}
