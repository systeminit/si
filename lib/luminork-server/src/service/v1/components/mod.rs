use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use dal::ComponentId;
use serde::Deserialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod get_component;
pub mod update_properties;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentsError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("diagram error: {0}")]
    Diagram(#[from] dal::diagram::DiagramError),
    #[error("prop error: {0}")]
    ManagementPrototype(#[from] dal::management::prototype::ManagementPrototypeError),
    #[error("prop error: {0}")]
    Prop(#[from] dal::prop::PropError),
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
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/:component_id",
        Router::new()
            .route("/", get(get_component::get_component))
            .route(
                "/properties",
                put(update_properties::update_component_properties),
            ),
    )
}
