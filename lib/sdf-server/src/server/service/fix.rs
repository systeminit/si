use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use thiserror::Error;

use dal::fix::FixError as DalFixError;
use dal::schema::SchemaError as DalSchemaError;
use dal::{
    ComponentError, ComponentId, FixResolverError, FuncBindingReturnValueError, StandardModelError,
    TransactionsError, UserError, UserPk, WorkflowRunnerError,
};

pub mod confirmations;
pub mod list;
pub mod run;

#[derive(Error, Debug)]
pub enum FixError {
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    FixResolver(#[from] FixResolverError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    DalFix(#[from] DalFixError),
    #[error(transparent)]
    DalSchema(#[from] DalSchemaError),
    #[error(transparent)]
    WorkflowRunner(#[from] WorkflowRunnerError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),

    #[error("no schema found for component {0}")]
    NoSchemaForComponent(ComponentId),
    #[error("no schema variant found for component {0}")]
    NoSchemaVariantForComponent(ComponentId),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
}

pub type FixResult<T> = std::result::Result<T, FixError>;

impl IntoResponse for FixError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/confirmations", get(confirmations::confirmations))
        .route("/list", get(list::list))
        .route("/run", post(run::run))
}
