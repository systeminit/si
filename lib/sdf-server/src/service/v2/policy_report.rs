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

mod fetch_batch;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PolicyReportError {
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] Box<dal::TransactionsError>),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("si db policy report error: {0}")]
    SiDbPolicyReport(#[from] si_db::PolicyReportError),
}

impl From<dal::TransactionsError> for PolicyReportError {
    fn from(value: dal::TransactionsError) -> Self {
        Box::new(value).into()
    }
}

pub(crate) type PolicyReportResult<T> = Result<T, PolicyReportError>;

impl IntoResponse for PolicyReportError {
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
    Router::new().route("/", get(fetch_batch::fetch_batch))
}
