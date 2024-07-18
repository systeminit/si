use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use si_frontend_types as frontend_types;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{server::state::AppState, service::ApiError};

mod contribute;
mod sync;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ModulesAPIError {
    #[error("axum http error: {0}")]
    AxumHttp(#[from] axum::http::Error),
    #[error("one or many module(s) were not contributed: {0:?}")]
    ContributionPartialFailure(
        Vec<(
            frontend_types::ModuleContributeRequestItem,
            module_index_client::ModuleIndexClientError,
        )>,
    ),
    #[error("all module(s) were not contributed: {0:?}")]
    ContributionTotalFailure(
        Vec<(
            frontend_types::ModuleContributeRequestItem,
            module_index_client::ModuleIndexClientError,
        )>,
    ),
    #[error("module error: {0}")]
    Module(#[from] dal::module::ModuleError),
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] module_index_client::ModuleIndexClientError),
    #[error("module index not configured")]
    ModuleIndexNotConfigured,
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
            Self::SchemaVariant(dal::SchemaVariantError::NotFound(schema_variant_id)) => {
                error!(%schema_variant_id, "schema variant not found");
                StatusCode::NOT_FOUND
            }
            Self::Module(dal::module::ModuleError::EmptyMetadata(_, _)) => StatusCode::BAD_REQUEST,
            Self::ContributionPartialFailure(_) => StatusCode::MULTI_STATUS,
            Self::ContributionTotalFailure(_) => StatusCode::BAD_GATEWAY,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/contribute", post(contribute::contribute))
        .route("/sync", get(sync::sync))
}
