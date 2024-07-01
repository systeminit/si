use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use dal::{
    func::{
        argument::FuncArgumentError,
        authoring::{FuncAuthoringClient, FuncAuthoringError},
        binding::FuncBindingsError,
    },
    ChangeSetError, DalContext, Func, FuncError, FuncId, WsEventError,
};
use si_frontend_types::FuncCode;
use thiserror::Error;

use crate::{server::state::AppState, service::ApiError};

pub mod argument;
pub mod binding;
pub mod create_func;
pub mod execute_func;
pub mod get_code;
pub mod list_funcs;
pub mod save_code;
pub mod test_execute;
pub mod update_func;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncAPIError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func bindings error: {0}")]
    FuncBindings(#[from] FuncBindingsError),
    #[error("The function name \"{0}\" is reserved")]
    FuncNameReserved(String),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("missing action kind")]
    MissingActionKindForActionFunc,
    #[error("missing action prototype")]
    MissingActionPrototype,
    #[error("missing func id")]
    MissingFuncId,
    #[error("no input location given")]
    MissingInputLocationForAttributeFunc,
    #[error("no output location given")]
    MissingOutputLocationForAttributeFunc,
    #[error("missing prototype id")]
    MissingPrototypeId,
    #[error("missing schema varianta and func id for leaf func")]
    MissingSchemaVariantAndFunc,
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("wrong function kind for binding")]
    WrongFunctionKindForBinding,
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}
pub type FuncAPIResult<T> = std::result::Result<T, FuncAPIError>;

impl IntoResponse for FuncAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            // these errors represent problems with the shape of the request
            Self::MissingActionKindForActionFunc
            | Self::MissingActionPrototype
            | Self::MissingFuncId
            | Self::MissingInputLocationForAttributeFunc
            | Self::MissingOutputLocationForAttributeFunc
            | Self::MissingPrototypeId
            | Self::MissingSchemaVariantAndFunc => StatusCode::BAD_REQUEST,
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
        .route("/", get(list_funcs::list_funcs))
        .route("/code", get(get_code::get_code)) // accepts a list of func_ids
        .route("/create", post(create_func::create_func))
        .route("/:func_id/update", post(update_func::update_func)) // only save the func's metadata
        .route("/:func_id/save_code", post(save_code::save_code)) // only saves func code
        .route("/:func_id/test_execute", post(test_execute::test_execute))
        .route("/:func_id/execute", post(execute_func::execute_func))
        // Func Bindings
        .route(
            "/:func_id/bindings/create",
            post(binding::create_binding::create_binding),
        )
        .route(
            "/:func_id/bindings/delete",
            post(binding::delete_binding::delete_binding),
        )
        .route(
            "/:func_id/bindings/update",
            post(binding::update_binding::update_binding),
        )
        // Attribute Bindings
        .route(
            "/:func_id/create_attribute_binding",
            post(binding::attribute::create_attribute_binding::create_attribute_binding),
        )
        .route(
            "/:func_id/reset_attribute_binding",
            post(binding::attribute::reset_attribute_binding::reset_attribute_binding),
        )
        .route(
            "/:func_id/update_attribute_binding",
            post(binding::attribute::update_attribute_binding::update_attribute_binding),
        )
        // Func Arguments
        .route(
            "/:func_id/:func_argument_id/update",
            post(argument::update_argument::update_func_argument),
        )
        .route(
            "/:func_id/create_argument",
            post(argument::create_argument::create_func_argument),
        )
        .route(
            "/:func_id/:func_argument_id/delete",
            post(argument::delete_argument::delete_func_argument),
        )
}

// helper to assemble the front end struct to return the code and types so SDF can decide when these events need to fire
pub async fn get_code_response(ctx: &DalContext, func_id: FuncId) -> FuncAPIResult<FuncCode> {
    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    let code = func.code_plaintext()?.unwrap_or("".to_string());
    Ok(FuncCode {
        func_id: func.id.into(),
        code: code.clone(),
        types: get_types(ctx, func_id).await?,
    })
}

// helper to get updated types to fire WSEvents so SDF can decide when these events need to fire
pub async fn get_types(ctx: &DalContext, func_id: FuncId) -> FuncAPIResult<String> {
    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    let types = [
        FuncAuthoringClient::compile_return_types(func.backend_response_type, func.backend_kind),
        FuncAuthoringClient::compile_types_from_bindings(ctx, func_id)
            .await?
            .as_str(),
        FuncAuthoringClient::compile_langjs_types(),
    ]
    .join("\n");
    Ok(types)
}
