use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use telemetry::prelude::*;
use thiserror::Error;

use dal::SchemaId;

use crate::{server::state::AppState, service::ApiError};

mod sync;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ModulesAPIError {
    #[error("too many latest modules for schema: {0} (at least two hashes found: {1} and {2})")]
    LatestModuleTooManyForSchema(SchemaId, String, String),
    #[error("module error: {0}")]
    Module(#[from] dal::module::ModuleError),
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] module_index_client::ModuleIndexClientError),
    #[error("module index not configured")]
    ModuleIndexNotConfigured,
    #[error("module missing schema id (module id: {0}) (module hash: {1})")]
    ModuleMissingSchemaId(String, String),
    #[error("module not found for schema: {0}")]
    ModuleNotFoundForSchema(SchemaId),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl IntoResponse for ModulesAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            Self::Transactions(dal::TransactionsError::ConflictsOccurred(_)) => {
                StatusCode::CONFLICT
            }
            Self::ModuleNotFoundForSchema(_)
            | Self::SchemaVariant(dal::SchemaVariantError::NotFound(_)) => StatusCode::NOT_FOUND,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new().route("/sync", get(sync::sync))
}
