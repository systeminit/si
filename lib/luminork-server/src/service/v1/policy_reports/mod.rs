#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::post,
};
use dal::WsEventError;
use thiserror::Error;

use super::common::ErrorIntoResponse;
use crate::AppState;

pub(crate) mod upload;

pub use upload::{
    UploadPolicyReportV1Request,
    UploadPolicyReportV1Response,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum PolicyReportsError {
    #[error("policy report error: {0}")]
    PolicyReport(#[from] si_db::PolicyReportError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

pub(crate) type PolicyReportsResult<T> = std::result::Result<T, PolicyReportsError>;

impl ErrorIntoResponse for PolicyReportsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
    }
}

impl IntoResponse for PolicyReportsError {
    fn into_response(self) -> Response {
        self.to_api_response()
    }
}

impl From<WsEventError> for PolicyReportsError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/", post(upload::upload_policy_report))
}
