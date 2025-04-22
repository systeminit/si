use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use dal::{
    ChangeSetError, SchemaVariantId, UserPk, WsEventError, cached_module::CachedModuleError,
    module::ModuleError,
};
use sdf_core::api_error::ApiError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::AppState;

pub mod create_unlocked_copy;
mod delete_unlocked_variant;
mod get_variant;
mod list_variants;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaVariantsAPIError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] CachedModuleError),
    #[error("cannot delete locked schema variant: {0}")]
    CannotDeleteLockedSchemaVariant(SchemaVariantId),
    #[error("cannot delete a schema variant that has attached components")]
    CannotDeleteVariantWithComponents,
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("invalid user: {0}")]
    InvalidUser(UserPk),
    #[error("Module error: {0}")]
    Module(#[from] ModuleError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] dal::schema::variant::authoring::VariantAuthoringError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaVariantsAPIResult<T> = std::result::Result<T, SchemaVariantsAPIError>;

impl IntoResponse for SchemaVariantsAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            Self::CannotDeleteVariantWithComponents | Self::CannotDeleteLockedSchemaVariant(_) => {
                StatusCode::PRECONDITION_FAILED
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
        .route("/", get(list_variants::list_variants))
        .route("/:schema_variant_id", get(get_variant::get_variant))
        .route(
            "/:schema_variant_id",
            post(create_unlocked_copy::create_unlocked_copy),
        )
        .route(
            "/:schema_variant_id",
            delete(delete_unlocked_variant::delete_unlocked_variant),
        )
}
