use std::result;

use axum::{http::StatusCode, response::IntoResponse, routing::post, Router};
use dal::ChangeSetId;
use thiserror::Error;

use crate::{service::ApiError, AppState};

mod apply;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("dvu roots are not empty for change set: {0}")]
    DvuRootsNotEmpty(ChangeSetId),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            Self::ChangeSetApply(_) => StatusCode::CONFLICT,
            Self::DvuRootsNotEmpty(_) => StatusCode::PRECONDITION_FAILED,
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub type ChangeSetAPIError = Error;

type Result<T> = result::Result<T, Error>;

pub fn v2_routes() -> Router<AppState> {
    Router::new().route("/apply", post(apply::apply))
}
