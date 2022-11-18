use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use thiserror::Error;

use dal::fix::FixError as DalFixError;
use dal::{
    ComponentError, ComponentId, ConfirmationPrototypeError, ConfirmationResolverError,
    ConfirmationResolverTreeError, FixBatchId, FixId, FixResolverError,
    FuncBindingReturnValueError, StandardModelError, TransactionsError, UserError, UserId,
    WorkflowRunnerError,
};

mod confirmations;
mod list;
mod recommendations;
mod run;

#[derive(Error, Debug)]
pub enum FixError {
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    ConfirmationResolver(#[from] ConfirmationResolverError),
    #[error(transparent)]
    ConfirmationResolverTree(#[from] ConfirmationResolverTreeError),
    #[error(transparent)]
    ConfirmationPrototype(#[from] ConfirmationPrototypeError),
    #[error(transparent)]
    FixResolver(#[from] FixResolverError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    DalFix(#[from] DalFixError),
    #[error(transparent)]
    WorkflowRunner(#[from] WorkflowRunnerError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),
    #[error("missing action for fix: {0}")]
    MissingAction(FixId),
    #[error("missing finished timestamp for fix: {0}")]
    MissingFinishedTimestampForFix(FixId),
    #[error("missing finished timestamp for fix batch: {0}")]
    MissingFinishedTimestampForFixBatch(FixBatchId),
    #[error("missing started timestamp for fix: {0}")]
    MissingStartedTimestampForFix(FixId),
    #[error("missing started timestamp for fix batch: {0}")]
    MissingStartedTimestampForFixBatch(FixBatchId),
    #[error("no schema found for component {0}")]
    NoSchemaForComponent(ComponentId),
    #[error("no schema variant found for component {0}")]
    NoSchemaVariantForComponent(ComponentId),
    #[error("invalid user {0}")]
    InvalidUser(UserId),
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
        .route("/recommendations", get(recommendations::recommendations))
        .route("/run", post(run::run))
}
