use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::func::argument::FuncArgumentError;
use dal::func::authoring::FuncAuthoringError;
use dal::func::summary::FuncSummaryError;
use dal::func::view::FuncViewError;
use dal::func::FuncAssociations;
use dal::input_sources::InputSourcesError;
use dal::schema::variant::SchemaVariantError;
use dal::{workspace_snapshot::WorkspaceSnapshotError, FuncId, TransactionsError};
use dal::{ChangeSetError, WsEventError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::server::{impl_default_error_into_response, state::AppState};

pub mod create_func;
pub mod delete_func;
pub mod get_func;
pub mod list_funcs;
pub mod list_input_sources;
pub mod save_and_exec;
pub mod save_func;
pub mod test_execute;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("context transaction error: {0}")]
    ContextTransaction(#[from] TransactionsError),
    #[error("dal func error: {0}")]
    Func(#[from] dal::func::FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func {0} cannot be converted to frontend variant")]
    FuncCannotBeTurnedIntoVariant(FuncId),
    #[error("The function name \"{0}\" is reserved")]
    FuncNameReserved(String),
    #[error("func summary error: {0}")]
    FuncSummary(#[from] FuncSummaryError),
    #[error("func view error: {0}")]
    FuncView(#[from] FuncViewError),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("input sources error: {0}")]
    InputSources(#[from] InputSourcesError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/create_func", post(create_func::create_func))
        .route("/delete_func", post(delete_func::delete_func))
        .route("/get_func", get(get_func::get_func))
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route(
            "/list_input_sources",
            get(list_input_sources::list_input_sources),
        )
        .route("/save_and_exec", post(save_and_exec::save_and_exec))
        .route("/save_func", post(save_func::save_func))
        .route("/test_execute", post(test_execute::execute))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SaveFuncResponse {
    types: String,
    associations: Option<FuncAssociations>,
}
