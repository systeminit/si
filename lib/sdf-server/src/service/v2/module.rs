use axum::{
    Router,
    extract::{
        DefaultBodyLimit,
        multipart::MultipartError,
    },
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use dal::{
    ChangeSetError,
    FuncError,
    WsEventError,
    cached_module::CachedModuleError,
    pkg::PkgError,
};
use sdf_core::api_error::ApiError;
use si_frontend_types as frontend_types;
use si_pkg::SiPkgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::AppState;

mod builtins;
mod contribute;
mod install_from_file;
mod list;
mod module_by_hash;
mod module_by_id;
mod sync;

// 20MB upload limit for module files
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 20;

pub type ModuleAPIResult<T> = Result<T, ModulesAPIError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ModulesAPIError {
    #[error("axum http error: {0}")]
    AxumHttp(#[from] axum::http::Error),
    #[error("cached module error: {0:?}")]
    CachedModule(#[from] CachedModuleError),
    #[error("changeset error: {0:?}")]
    Changeset(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("module not contributed: {0:?}")]
    ContributionFailure(frontend_types::ModuleContributeRequest),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("module error: {0}")]
    Module(#[from] dal::module::ModuleError),
    #[error("Module hash not be found: {0}")]
    ModuleHashNotFound(String),
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] module_index_client::ModuleIndexClientError),
    #[error("module index not configured")]
    ModuleIndexNotConfigured,
    #[error("multipart error: {0}")]
    Multipart(#[from] MultipartError),
    #[error("pkg error: {0:?}")]
    Pkg(#[from] PkgError),
    #[error("pkg file error: {0}")]
    PkgFileError(&'static str),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("changeset error: {0:?}")]
    Serde(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("si pkg error: {0}")]
    SiPkg(#[from] SiPkgError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] dal::schema::variant::authoring::VariantAuthoringError),
    #[error("WsEvent error: {0}")]
    WsEvent(#[from] WsEventError),
}

impl IntoResponse for ModulesAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            Self::SchemaVariant(dal::SchemaVariantError::NotFound(schema_variant_id)) => {
                error!(%schema_variant_id, "schema variant not found");
                StatusCode::NOT_FOUND
            }
            Self::Module(dal::module::ModuleError::EmptyMetadata(_, _)) => StatusCode::BAD_REQUEST,
            Self::ContributionFailure(_) => StatusCode::BAD_REQUEST,
            Self::ModuleHashNotFound(_) => StatusCode::NOT_FOUND,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/contribute", post(contribute::contribute))
        .route("/sync", get(sync::sync))
        .route("/", get(list::list))
        .route("/:module_id/builtins/reject", post(builtins::reject))
        .route("/:module_id/builtins/promote", post(builtins::promote))
        .route("/module_by_hash", get(module_by_hash::module_by_hash))
        .route("/module_by_id", get(module_by_id::remote_module_by_id))
        .route(
            "/install_from_file",
            post(install_from_file::install_module_from_file),
        )
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
}
