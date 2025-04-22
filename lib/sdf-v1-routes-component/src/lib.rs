use std::num::ParseIntError;

use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use dal::slow_rt::SlowRuntimeError;
use dal::validation::ValidationError;
use dal::{ChangeSetError, TransactionsError};
use dal::{
    ComponentError as DalComponentError, FuncError, StandardModelError, WorkspaceError,
    WorkspaceSnapshotError, action::ActionError, action::prototype::ActionPrototypeError,
};
use dal::{
    PropId, SchemaVariantError, SecretError as DalSecretError, WsEventError,
    attribute::value::debug::AttributeDebugViewError, component::ComponentId,
};
use dal::{attribute::value::AttributeValueError, component::debug::ComponentDebugViewError};
use dal::{prop::PropError, socket::output::OutputSocketError};
use dal::{property_editor::PropertyEditorError, socket::input::InputSocketError};
use sdf_core::{api_error::ApiError, app_state::AppState};
use si_posthog::PosthogError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

use crate::conflicts_for_component::conflicts_for_component;

mod autoconnect;
pub mod conflicts_for_component;
pub mod debug;
pub mod delete_property_editor_value;
pub mod get_actions;
pub mod get_code;
pub mod get_diff;
pub mod get_property_editor_schema;
pub mod get_property_editor_values;
pub mod get_resource;
pub mod insert_property_editor_value;
pub mod json;
pub mod list_qualifications;
mod manage;
mod override_with_connection;
pub mod refresh;
pub mod restore_default_function;
pub mod set_name;
pub mod set_resource_id;
pub mod set_type;
mod unmanage;
pub mod update_property_editor_value;
mod upgrade;

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
    Diagram(#[from] sdf_v1_routes_diagram::DiagramError),
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
    StandardModel(#[from] StandardModelError),
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
        .route(
            "/restore_default_function",
            post(restore_default_function::restore_default_function),
        )
        .route("/set_type", post(set_type::set_type))
        .route("/set_name", post(set_name::set_name))
        .route("/set_resource_id", post(set_resource_id::set_resource_id))
        .route("/refresh", post(refresh::refresh))
        .route("/debug", get(debug::debug_component))
        .route("/autoconnect", post(autoconnect::autoconnect))
        .route(
            "/override_with_connection",
            post(override_with_connection::override_with_connection),
        )
        .route("/json", get(json::json))
        .route("/upgrade_component", post(upgrade::upgrade))
        .route("/conflicts", get(conflicts_for_component))
        .route("/manage", post(manage::manage))
        .route("/unmanage", post(unmanage::unmanage))
}
