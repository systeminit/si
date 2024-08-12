use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::put,
    Router,
};
use dal::func::runner::FuncRunnerError;
use thiserror::Error;

use crate::server::error;
use crate::server::state::AppState;
use crate::service::ApiError;

mod cancel_execution;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AdminAPIError {
    #[error("func runner error: {0}")]
    FuncRunner(#[from] FuncRunnerError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
}

impl IntoResponse for AdminAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            AdminAPIError::FuncRunner(FuncRunnerError::DoNotHavePermissionToCancelExecution) => {
                StatusCode::UNAUTHORIZED
            }
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };
        error!(si.error.message = ?self.to_string());

        ApiError::new(status_code, self).into_response()
    }
}

pub type AdminAPIResult<T> = Result<T, AdminAPIError>;

pub fn v2_routes() -> Router<AppState> {
    Router::new().route(
        "/func/runs/:func_run_id/cancel_execution",
        put(cancel_execution::cancel_execution),
    )
}
