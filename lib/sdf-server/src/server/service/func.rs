use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::func::authoring::FuncAuthoringError;
use dal::func::summary::FuncSummaryError;
use dal::func::view::FuncViewError;
use dal::input_sources::InputSourcesError;
use dal::schema::variant::SchemaVariantError;
use dal::{attribute::prototype::AttributePrototypeError, func::argument::FuncArgumentError};
use dal::{workspace_snapshot::WorkspaceSnapshotError, FuncId, TransactionsError};
use dal::{ChangeSetError, WsEventError};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use crate::server::{impl_default_error_into_response, state::AppState};

pub mod create_attribute_prototype;
pub mod create_func;
pub mod create_func_argument;
pub mod delete_func;
pub mod delete_func_argument;
pub mod get_func;
pub mod get_func_run;
pub mod list_func_arguments;
pub mod list_funcs;
pub mod list_input_sources;
pub mod remove_attribute_prototype;
pub mod save_and_exec;
pub mod save_func;
pub mod test_execute;
pub mod update_attribute_prototype;
pub mod update_func_argument;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
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
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transaction error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/create_attribute_prototype",
            post(create_attribute_prototype::create_attribute_prototype),
        )
        .route("/create_func", post(create_func::create_func))
        .route(
            "/create_func_argument",
            post(create_func_argument::create_func_argument),
        )
        .route("/delete_func", post(delete_func::delete_func))
        .route(
            "/delete_func_argument",
            post(delete_func_argument::delete_func_argument),
        )
        .route("/get_func", get(get_func::get_func))
        .route("/get_func_run", get(get_func_run::get_func_run))
        .route(
            "/list_func_arguments",
            get(list_func_arguments::list_func_arguments),
        )
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route(
            "/list_input_sources",
            get(list_input_sources::list_input_sources),
        )
        .route(
            "/remove_attribute_prototype",
            post(remove_attribute_prototype::remove_attribute_prototype),
        )
        .route("/save_and_exec", post(save_and_exec::save_and_exec))
        .route("/save_func", post(save_func::save_func))
        .route("/test_execute", post(test_execute::test_execute))
        .route(
            "/update_attribute_prototype",
            post(update_attribute_prototype::update_attribute_prototype),
        )
        .route(
            "/update_func_argument",
            post(update_func_argument::update_func_argument),
        )
}
