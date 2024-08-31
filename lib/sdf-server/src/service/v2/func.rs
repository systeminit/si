use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use dal::{
    attribute::value::AttributeValueError,
    func::{
        argument::FuncArgumentError, authoring::FuncAuthoringError, binding::FuncBindingError,
        runner::FuncRunnerError,
    },
    ChangeSetError, DalContext, Func, FuncError, FuncId, WsEventError,
};
use si_frontend_types::FuncCode;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::FunctionResultFailureErrorKind;

use crate::{service::ApiError, AppState};

pub mod argument;
pub mod binding;
pub mod create_func;
pub mod create_unlocked_copy;
pub mod delete_func;
pub mod execute_func;
pub mod get_code;
pub mod get_func_run;
pub mod list_all_funcs;
pub mod list_funcs;
pub mod save_code;
pub mod test_execute;
pub mod update_func;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncAPIError {
    #[error("cannot delete binding for func kind")]
    CannotDeleteBindingForFunc,
    #[error("cannot delete locked func: {0}")]
    CannotDeleteLockedFunc(FuncId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func already unlocked: {0}")]
    FuncAlreadyUnlocked(FuncId),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func bindings error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("The function name \"{0}\" is reserved")]
    FuncNameReserved(String),
    #[error("The function does not exist")]
    FuncNotFound(FuncId),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
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
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshotError(#[from] dal::WorkspaceSnapshotError),
    #[error("wrong function kind for binding")]
    WrongFunctionKindForBinding,
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}
pub type FuncAPIResult<T> = Result<T, FuncAPIError>;

impl IntoResponse for FuncAPIError {
    fn into_response(self) -> Response {
        let err_string = self.to_string();

        fn func_runner_err_to_status_and_message(
            err: FuncRunnerError,
        ) -> (StatusCode, Option<String>) {
            match err {
                FuncRunnerError::ResultFailure { kind, message, .. } => match kind {
                    FunctionResultFailureErrorKind::VeritechServer => {
                        (ApiError::DEFAULT_ERROR_STATUS_CODE, None)
                    }
                    FunctionResultFailureErrorKind::InvalidReturnType
                    | FunctionResultFailureErrorKind::KilledExecution
                    | FunctionResultFailureErrorKind::ActionFieldWrongType => {
                        (StatusCode::UNPROCESSABLE_ENTITY, Some(message))
                    }
                    FunctionResultFailureErrorKind::UserCodeException(lang_server_error_kind) => {
                        let err = format!("{lang_server_error_kind}: {message}");
                        (StatusCode::UNPROCESSABLE_ENTITY, Some(err))
                    }
                },
                _ => (ApiError::DEFAULT_ERROR_STATUS_CODE, None),
            }
        }

        let (status_code, maybe_message) = match self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                (StatusCode::FORBIDDEN, None)
            }
            // these errors represent problems with the shape of the request
            Self::MissingActionKindForActionFunc
            | Self::MissingActionPrototype
            | Self::MissingFuncId
            | Self::MissingInputLocationForAttributeFunc
            | Self::MissingOutputLocationForAttributeFunc
            | Self::MissingPrototypeId
            | Self::MissingSchemaVariantAndFunc
            | Self::Func(FuncError::FuncLocked(_))
            | Self::SchemaVariant(dal::SchemaVariantError::SchemaVariantLocked(_)) => {
                (StatusCode::BAD_REQUEST, None)
            }

            // Return 404 when the func is not found
            Self::FuncNotFound(_) |
            // When a graph node cannot be found for a schema variant, it is not found
            Self::SchemaVariant(dal::SchemaVariantError::NotFound(_)) => (StatusCode::NOT_FOUND, None),

            // When the authoring changes requested would result in a cycle
            Self::FuncBinding(FuncBindingError::SchemaVariant(
                dal::SchemaVariantError::AttributePrototypeArgument(
                    dal::attribute::prototype::argument::AttributePrototypeArgumentError::WorkspaceSnapshot(
                        dal::WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                            dal::workspace_snapshot::graph::WorkspaceSnapshotGraphError::CreateGraphCycle,
                        ),
                    ),
                ),
            )) => (StatusCode::UNPROCESSABLE_ENTITY, None),

            // Return 422 if the error is related to the user code being invalid
            Self::FuncAuthoring(FuncAuthoringError::AttributeValue(AttributeValueError::FuncRunner(err))) =>
                func_runner_err_to_status_and_message(*err),
            Self::FuncAuthoring(FuncAuthoringError::FuncRunner(err)) => func_runner_err_to_status_and_message(err),


            _ => (ApiError::DEFAULT_ERROR_STATUS_CODE, None)
        };

        error!(si.error.message = ?err_string);

        ApiError::new(status_code, maybe_message.unwrap_or(err_string)).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        // Func Stuff
        .route("/", get(list_funcs::list_funcs))
        .route("/including_pruned", get(list_all_funcs::list_all_funcs))
        .route("/code", get(get_code::get_code)) // accepts a list of func_ids
        .route("/runs/:func_run_id", get(get_func_run::get_func_run)) // accepts a list of func_ids
        .route("/", post(create_func::create_func))
        .route("/:func_id", put(update_func::update_func)) // only save the func's metadata
        .route("/:func_id/code", put(save_code::save_code)) // only saves func code
        .route("/:func_id/test_execute", post(test_execute::test_execute))
        .route("/:func_id/execute", post(execute_func::execute_func))
        .route(
            "/:func_id",
            post(create_unlocked_copy::create_unlocked_copy),
        )
        .route("/:func_id", delete(delete_func::delete_func))
        // Func Bindings
        .route(
            "/:func_id/bindings",
            post(binding::create_binding::create_binding),
        )
        .route(
            "/:func_id/bindings",
            delete(binding::delete_binding::delete_binding),
        )
        .route(
            "/:func_id/bindings",
            put(binding::update_binding::update_binding),
        )
        // Reset Attribute Bindings
        .route(
            "/:func_id/reset_attribute_binding",
            post(binding::attribute::reset_attribute_binding::reset_attribute_binding),
        )
        // Func Arguments
        .route(
            "/:func_id/arguments",
            post(argument::create_argument::create_func_argument),
        )
        .route(
            "/:func_id/arguments/:func_argument_id",
            put(argument::update_argument::update_func_argument),
        )
        .route(
            "/:func_id/arguments/:func_argument_id",
            delete(argument::delete_argument::delete_func_argument),
        )
}

// helper to assemble the front end struct to return the code and types so SDF can decide when these events need to fire
pub async fn get_code_response(ctx: &DalContext, func_id: FuncId) -> FuncAPIResult<FuncCode> {
    let func = Func::get_by_id(ctx, func_id)
        .await?
        .ok_or(FuncAPIError::FuncNotFound(func_id))?;
    let code = func.code_plaintext()?.unwrap_or("".to_string());
    Ok(FuncCode {
        func_id: func.id.into(),
        code: code.clone(),
    })
}
