use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use thiserror::Error;

use dal::schema::SchemaError as DalSchemaError;
use dal::{
    func::binding_return_value::FuncBindingReturnValueError, ActionBatchError, ActionRunnerError,
    ComponentError, ComponentId, StandardModelError, TransactionsError, UserError, UserPk,
};

use crate::server::state::AppState;

pub mod history;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionError {
    #[error(transparent)]
    ActionBatch(#[from] ActionBatchError),
    #[error(transparent)]
    ActionRunner(#[from] ActionRunnerError),
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
    Router::new().route("/history", get(history::history))
}
