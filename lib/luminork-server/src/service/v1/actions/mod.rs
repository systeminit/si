use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        get,
        post,
    },
};
use serde::Deserialize;
use si_db::SiDbError;
use si_id::ActionId;
use si_layer_cache::LayerDbError;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod cancel_action;
pub mod get_actions;
pub mod put_on_hold;
pub mod retry_action;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ActionsError {
    #[error("actions error: {0}")]
    Action(#[from] dal::action::ActionError),
    #[error("action not found: {0}")]
    ActionNotFound(ActionId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] dal::action::prototype::ActionPrototypeError),
    #[error("funcs error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("Cannot update action state that's not Queued to On Hold. Action with Id {0}")]
    InvalidOnHoldTransition(ActionId),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("SI db error: {0}")]
    SiDb(#[from] SiDbError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("wsevent error: {0}")]
    WsEventError(#[from] dal::WsEventError),
}

pub type ActionsResult<T> = Result<T, ActionsError>;

#[derive(Deserialize, ToSchema)]
pub struct ActionV1RequestPath {
    #[schema(value_type = String)]
    pub action_id: ActionId,
}

impl IntoResponse for ActionsError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl From<JsonRejection> for ActionsError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                ActionsError::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                ActionsError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => ActionsError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => ActionsError::Validation(format!("JSON validation error: {rejection}")),
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for ActionsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ActionsError::ActionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ActionsError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            ActionsError::InvalidOnHoldTransition(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_actions::get_actions))
        .nest(
            "/:action_id",
            Router::new()
                .route("/cancel", post(cancel_action::cancel_action))
                .route("/retry", post(retry_action::retry_action))
                .route("/put_on_hold", post(put_on_hold::put_on_hold)),
        )
}
