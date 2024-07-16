use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use dal::{ChangeSetError, SchemaVariantId};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{server::state::AppState, service::ApiError};

mod delete_unlocked_variant;
mod get_variant;
mod list_variants;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaVariantsAPIError {
    #[error("cannot delete locked schema variant: {0}")]
    CannotDeleteLockedSchemaVariant(SchemaVariantId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("hyper error: {0}")]
    Http(#[from] axum::http::Error),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
}

pub type SchemaVariantsAPIResult<T> = std::result::Result<T, SchemaVariantsAPIError>;

impl IntoResponse for SchemaVariantsAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            // Return 409 when we see a conflict
            Self::Transactions(dal::TransactionsError::ConflictsOccurred(_)) => {
                StatusCode::CONFLICT
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
            "/:schema_variant_id/delete_unlocked_copy",
            post(delete_unlocked_variant::delete_unlocked_variant),
        )
}
