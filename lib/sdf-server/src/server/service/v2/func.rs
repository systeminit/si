use axum::{
    extract::{OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use dal::{
    func::authoring::FuncAuthoringError, schema::variant, ChangeSetError, ChangeSetId, FuncError, Schema, SchemaVariant, SchemaVariantId, WorkspacePk
};
use si_frontend_types as frontend_types;
use thiserror::Error;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    state::AppState,
    tracking::track,
};

pub mod argument;
pub mod attribute_binding;
pub mod binding;
pub mod func;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncAPIError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError)
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
}
pub type FuncAPIResult<T> = std::result::Result<T, FuncAPIError>;

impl IntoResponse for FuncAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            // When a graph node cannot be found for a schema variant, it is not found
            Self::SchemaVariant(dal::SchemaVariantError::NotFound(_)) => StatusCode::NOT_FOUND,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        // Func Stuff
        .route("/", get(func::list_funcs))
        .route("/code", post(func::get_code)) // accepts a list of func_ids
        .route("/create", post(func::create))
        .route("/:func_id/update", post(func::update)) // only save the func's metadata
        .route("/:func_id/save_code", post(func::save_code)) // only saves func code
        // .route("/:func_id/test_execute") todo
        //.route("/:func_id/execute") todo
        // Func Bindings
        .route("/bindings", get(binding::list_bindings)) // accepts a list of func_ids
        .route("/:func_id/bindings/create", post(binding::create_binding))
        .route("/:func_id/bindings/delete", post(binding::delete_binding))
        .route("/:func_id/bindings/update", post(binding::update_binding))
        // Attribute Bindings
        .route(
            "/:func_id/create_attribute_binding",
            post(attribute_binding::create_attribute_binding),
        )
        .route(
            "/:func_id/reset_attribute_binding",
            post(attribute_binding::reset_attribute_binding),
        )
        .route(
            "/:func_id/update_attribute_binding",
            post(attribute_binding::update_attribute_binding),
        )
        // Func Arguments
        .route("/:func_id/arguments/", get(v2::argument::list_arguments))
        .route(
            "/:func_id/:argument_id/update",
            post(argument::update_func_argument),
        )
        .route(
            "/:func_id/create_argument",
            post(argument::create_func_argument),
        )
        .route(
            "/:func_id/delete_argument",
            put(argument::delete_func_argument),
        )
}
