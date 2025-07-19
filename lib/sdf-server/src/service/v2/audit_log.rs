use axum::{
    Router,
    response::{
        IntoResponse,
        Response,
    },
    routing::get,
};
use sdf_core::api_error::ApiError;
use thiserror::Error;

use crate::AppState;

mod list_audit_logs;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLogError {
    #[error("dal audit logging error: {0}")]
    DalAuditLogging(#[from] Box<dal::audit_logging::AuditLoggingError>),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] Box<dal::ChangeSetError>),
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] Box<dal::TransactionsError>),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
}

impl From<dal::audit_logging::AuditLoggingError> for AuditLogError {
    fn from(value: dal::audit_logging::AuditLoggingError) -> Self {
        Box::new(value).into()
    }
}

impl From<dal::ChangeSetError> for AuditLogError {
    fn from(value: dal::ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<dal::TransactionsError> for AuditLogError {
    fn from(value: dal::TransactionsError) -> Self {
        Box::new(value).into()
    }
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
    Router::new().route("/", get(list_audit_logs::list_audit_logs))
}
