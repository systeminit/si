use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use dal::{
    func::binding_return_value::FuncBindingReturnValueError, ComponentError, ComponentId,
    FuncBindingError, FuncBindingId, FuncId, SchemaId, SchemaVariantId, StandardModelError,
    TransactionsError, WorkflowPrototypeError, WorkflowPrototypeId, WorkflowRunnerError,
    WorkflowRunnerId,
};

use thiserror::Error;

mod history;
mod info;
mod list;
mod resolve;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Workflow(#[from] dal::WorkflowError),
    #[error(transparent)]
    WorkflowRunner(#[from] WorkflowRunnerError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("prototype not found {0}")]
    PrototypeNotFound(WorkflowPrototypeId),
    #[error("function not found {0}")]
    FuncNotFound(FuncId),
    #[error("function binding not found {0}")]
    FuncBindingNotFound(FuncBindingId),
    #[error(transparent)]
    WorkflowPrototype(#[from] WorkflowPrototypeError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound(ComponentId),
    #[error("component name not found")]
    ComponentNameNotFound(ComponentId),
    #[error("schema not found")]
    SchemaNotFound(SchemaId),
    #[error("schema variant not found")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("runner not found")]
    RunnerNotFound(WorkflowRunnerId),
    #[error("runner state not found for runner id: {0}")]
    RunnerStateNotFound(WorkflowRunnerId),
}

pub type WorkflowResult<T> = std::result::Result<T, WorkflowError>;

impl IntoResponse for WorkflowError {
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
        .route("/list", get(list::list))
        .route("/resolve", post(resolve::resolve))
        .route("/history", get(history::history))
        .route("/info", get(info::info))
}
