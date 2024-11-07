use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use thiserror::Error;

use crate::{service::ApiError, AppState};

mod get_filter_options;
mod list_audit_logs;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLogError {
    #[error("dal audit logging error: {0}")]
    DalAuditLogging(#[from] dal::audit_logging::AuditLoggingError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] dal::TransactionsError),
    #[error("dal user error: {0}")]
    DalUser(#[from] dal::UserError),
}

pub type AuditLogResult<T> = Result<T, AuditLogError>;

impl IntoResponse for AuditLogError {
    fn into_response(self) -> Response {
        let err_string = self.to_string();

        #[allow(clippy::match_single_binding)]
        let (status_code, maybe_message) = match self {
            _ => (ApiError::DEFAULT_ERROR_STATUS_CODE, None),
        };

        ApiError::new(status_code, maybe_message.unwrap_or(err_string)).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_audit_logs::list_audit_logs))
        .route("/filters", get(get_filter_options::get_filter_options))
}
