use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
};
use dal::{
    AttributeValueId,
    KeyPairError,
    prop::PropError,
};
use sdf_core::api_error::ApiError;
use serde::Deserialize;
use si_id::ComponentId;

use crate::app_state::AppState;

pub mod attributes;
pub mod name;
pub mod secrets;

#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("attribute $source: {0} has extra fields: {1}")]
    AttributeSourceHasExtraFields(serde_json::Value, serde_json::Value),
    #[error("invalid attribute $source: {0}")]
    AttributeSourceInvalid(serde_json::Value),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("attribute value {0} not from component {1}")]
    AttributeValueNotFromComponent(AttributeValueId, ComponentId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("dal secret error: {0}")]
    DalSecret(#[from] dal::SecretError),
    #[error("json pointer parse error: {0}")]
    JsonptrParseError(#[from] jsonptr::ParseError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("no value to set at path {0}")]
    NoValueToSet(String),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("source component not found: {0}")]
    SourceComponentNotFound(String),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match self {
            Error::AttributeSourceHasExtraFields(..)
            | Error::AttributeSourceInvalid(..)
            | Error::NoValueToSet(..)
            | Error::SourceComponentNotFound(..) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_message = self.to_string();
        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new().nest(
        "/:componentId",
        Router::new()
            .nest("/attributes", attributes::v2_routes())
            .nest("/name", name::v2_routes())
            .nest("/secret", secrets::v2_routes()),
    )
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
struct ComponentIdFromPath {
    component_id: ComponentId,
}
