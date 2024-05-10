use axum::routing::post;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use thiserror::Error;

use dal::{
    action::prototype::ActionPrototypeError, schema::SchemaError as DalSchemaError, ActionId,
};
use dal::{
    func::binding::return_value::FuncBindingReturnValueError, ComponentError, ComponentId,
    DeprecatedActionBatchError, DeprecatedActionRunnerError, StandardModelError, TransactionsError,
    UserError, UserPk,
};

use crate::server::state::AppState;

mod cancel;
pub mod history;
pub mod list_actions;
mod put_on_hold;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionError {
    #[error(transparent)]
    Action(#[from] dal::action::ActionError),
    #[error(transparent)]
    ActionBatch(#[from] DeprecatedActionBatchError),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    ActionRunner(#[from] DeprecatedActionRunnerError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),
    // #[error(transparent)]
    // DalFix(#[from] DalFixError),
    #[error(transparent)]
    DalSchema(#[from] DalSchemaError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("Cannot cancel Running or Dispatched actions. ActionId {0}")]
    InvalidActionCancellation(ActionId),
    #[error("Cannot update action state that's not Queued to On Hold. Action with Id {0}")]
    InvalidOnHoldTransition(ActionId),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("no schema found for component {0}")]
    NoSchemaForComponent(ComponentId),
    #[error("no schema variant found for component {0}")]
    NoSchemaVariantForComponent(ComponentId),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    User(#[from] UserError),
}

pub type ActionResult<T> = std::result::Result<T, ActionError>;

impl IntoResponse for ActionError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/history", get(history::history))
        .route("/list", get(list_actions::list_actions))
        .route("/put_on_hold", post(put_on_hold::put_on_hold))
        .route("/cancel", post(cancel::cancel))
}
