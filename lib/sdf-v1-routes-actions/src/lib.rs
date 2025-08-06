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
    ComponentError,
    ComponentId,
    FuncError as DalFuncError,
    TransactionsError,
    UserPk,
    WsEventError,
    action::{
        ActionId,
        prototype::ActionPrototypeError,
    },
    schema::SchemaError as DalSchemaError,
};
use sdf_core::{
    api_error::ApiError,
    app_state::AppState,
};
use si_layer_cache::LayerDbError;
use thiserror::Error;

mod cancel;
mod history;
pub mod list_actions;
mod put_on_hold;
mod retry;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionError {
    #[error(transparent)]
    Action(#[from] dal::action::ActionError),
    #[error("action history is missing a field - this is a bug!: {0}")]
    ActionHistoryFieldMissing(String),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),
    #[error(transparent)]
    DalSchema(#[from] DalSchemaError),
    #[error(transparent)]
    Func(#[from] DalFuncError),
    #[error("Cannot cancel Running or Dispatched actions. ActionId {0}")]
    InvalidActionCancellation(ActionId),
    #[error("Cannot update action state that's not Queued to On Hold. Action with Id {0}")]
    InvalidOnHoldTransition(ActionId),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("no schema found for component {0}")]
    NoSchemaForComponent(ComponentId),
    #[error("no schema variant found for component {0}")]
    NoSchemaVariantForComponent(ComponentId),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("wsevent error: {0}")]
    WsEventError(#[from] WsEventError),
}

pub type ActionResult<T> = std::result::Result<T, ActionError>;

impl IntoResponse for ActionError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ActionError::InvalidOnHoldTransition(_) => (StatusCode::NOT_MODIFIED, self.to_string()),
            ActionError::Action(dal::action::ActionError::WorkspaceSnapshot(err))
                if err.is_node_with_id_not_found() =>
            {
                (StatusCode::GONE, err.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_actions::list_actions)) // USED IN OLD UI
        .route("/put_on_hold", post(put_on_hold::put_on_hold)) // USED IN OLD UI
        .route("/cancel", post(cancel::cancel)) // USED IN OLD UI
        .route("/retry", post(retry::retry)) // USED IN OLD UI
        .route("/history", get(history::history)) // USED IN OLD UI
}
