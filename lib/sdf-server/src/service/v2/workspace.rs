use std::time::Duration;

use axum::{
    Router,
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
use dal::WorkspacePk;
use sdf_core::{
    api_error::ApiError,
    index::IndexResult,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

use super::AccessBuilder;
use crate::app_state::AppState;

mod get_deployment_index;
mod install_workspace;
mod list_workspace_users;
mod mjolnir;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceAPIError {
    #[error("semaphore acquire error: {0}")]
    Acquire(#[from] tokio::sync::AcquireError),
    #[error("deserializing mv index data error: {0}")]
    DeserializingMvIndexData(#[source] serde_json::Error),
    #[error("edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("frigg error: {0}")]
    Frigg(#[from] frigg::FriggError),
    #[error("deployment index not found")]
    IndexNotFound,
    #[error("latest item not found; workspace_id={0}, kind={1}, id={2}")]
    LatestItemNotFound(WorkspacePk, String, String),
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] module_index_client::ModuleIndexClientError),
    #[error("module index url not set")]
    ModuleIndexUrlNotSet,
    #[error("cannot export workspace using root tenancy")]
    RootTenancyExportAttempt,
    #[error("cannot install workspace using root tenancy")]
    RootTenancyInstallAttempt,
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::SiDbError),
    #[error("tokio task join error: {0}")]
    TokioJoin(#[from] JoinError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("unable to parse url: {0}")]
    Url(#[from] url::ParseError),
    #[error("timed out when watching index with duration: {0:?}")]
    WatchIndexTimeout(Duration),
    #[error("workspace error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
}

pub type WorkspaceAPIResult<T> = Result<T, WorkspaceAPIError>;

impl IntoResponse for WorkspaceAPIError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            WorkspaceAPIError::LatestItemNotFound(_, _, _)
            | Self::Workspace(dal::WorkspaceError::WorkspaceNotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/install", post(install_workspace::install_workspace))
        .route("/users", get(list_workspace_users::list_workspace_users))
        .route(
            "/deployment_index",
            get(get_deployment_index::get_deployment_index),
        )
        .route(
            "/multi_mjolnir",
            post(mjolnir::get_multiple_front_end_objects),
        )
        .route("/mjolnir", post(mjolnir::get_front_end_object))
}
