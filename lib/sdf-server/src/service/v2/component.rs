use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        delete,
        post,
        put,
    },
};
use dal::{
    KeyPairError,
    func::authoring::FuncAuthoringError,
    prop::PropError,
};
use sdf_core::api_error::ApiError;
use serde::Deserialize;
use si_id::ComponentId;

use crate::app_state::AppState;

pub mod attributes;
pub mod delete_components;
pub mod name;
pub mod restore_components;
pub mod secrets;
pub mod upgrade_components;

#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("action error: {0}")]
    Action(#[from] dal::action::ActionError),
    #[error("attributes error: {0}")]
    Attributes(#[from] dal::attribute::attributes::Error),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("dal secret error: {0}")]
    DalSecret(#[from] dal::SecretError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("json pointer parse error: {0}")]
    JsonptrParseError(#[from] jsonptr::ParseError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("no value to set at path {0}")]
    NoValueToSet(String),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("schema variant upgrade not required")]
    SchemaVariantUpgradeSkipped,
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("component upgrade skipped due to running or dispatched actions")]
    UpgradeSkippedDueToActions,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match self {
            Error::NoValueToSet(..) => StatusCode::BAD_REQUEST,
            Error::SchemaVariantUpgradeSkipped | Error::UpgradeSkippedDueToActions => {
                StatusCode::NOT_MODIFIED
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_message = self.to_string();
        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/upgrade", post(upgrade_components::upgrade_components))
        .route("/delete", delete(delete_components::delete_components))
        .route("/restore", put(restore_components::restore_components))
        .nest(
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
