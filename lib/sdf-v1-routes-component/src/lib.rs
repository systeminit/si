use std::num::ParseIntError;

use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use dal::{
    ChangeSetError,
    ComponentError as DalComponentError,
    FuncError,
    PropId,
    SchemaVariantError,
    SecretError as DalSecretError,
    TransactionsError,
    WorkspaceError,
    WorkspaceSnapshotError,
    WsEventError,
    action::{
        ActionError,
        prototype::ActionPrototypeError,
    },
    attribute::value::{
        AttributeValueError,
        debug::AttributeDebugViewError,
    },
    component::{
        ComponentId,
        debug::ComponentDebugViewError,
    },
    prop::PropError,
    property_editor::PropertyEditorError,
    slow_rt::SlowRuntimeError,
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    validation::ValidationError,
};
use sdf_core::{
    api_error::ApiError,
    app_state::AppState,
};
use si_posthog::PosthogError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

pub mod json;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
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
    #[error("diagram error: {0}")]
    DiagramError(#[from] dal::diagram::DiagramError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("invalid visibility")]
    InvalidVisibility,
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("key {0} already exists for that map")]
    KeyAlreadyExists(String),
    #[error("component not found for id: {0}")]
    NotFound(ComponentId),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("posthog error: {0}")]
    Posthog(#[from] PosthogError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error("property editor error: {0}")]
    PropertyEditor(#[from] PropertyEditorError),
    #[error("prop not found for id: {0}")]
    PropNotFound(PropId),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("schema variant upgrade not required")]
    SchemaVariantUpgradeSkipped,
    #[error("dal secret error: {0}")]
    Secret(#[from] DalSecretError),
    #[error("secret id deserialization error: {0}")]
    SecretIdDeserialization(#[source] serde_json::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("component upgrade skipped due to running or dispatched actions")]
    UpgradeSkippedDueToActions,
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

impl IntoResponse for ComponentError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ComponentError::SchemaNotFound
            | ComponentError::InvalidVisibility
            | ComponentError::PropNotFound(_)
            | ComponentError::SchemaVariantNotFound
            | ComponentError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ComponentError::PropertyEditor(err) => match err {
                PropertyEditorError::ComponentNotFound
                | PropertyEditorError::SchemaVariantNotFound(_) => {
                    (StatusCode::NOT_FOUND, err.to_string())
                }

                _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            },
            ComponentError::SchemaVariantUpgradeSkipped => {
                (StatusCode::NOT_MODIFIED, self.to_string())
            }
            ComponentError::KeyAlreadyExists(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            ComponentError::DalComponent(err) => match err {
                DalComponentError::NotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            },
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/json", get(json::json)) // USED IN FUNC EDITOR
}
