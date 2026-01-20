use axum::{
    Router,
    response::{
        IntoResponse,
        Response,
    },
    routing::get,
};
use sdf_core::api_error::ApiError;
use serde::Serialize;
use si_db::PolicyReport;
use si_id::{
    ChangeSetId,
    PolicyReportId,
    WorkspacePk,
};
use thiserror::Error;

use crate::AppState;

mod fetch_batch;
mod grouped_by_name;

// NOTE(nick): we need this to convert to camelcase and to handle timestamp conversion.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyReportView {
    id: PolicyReportId,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    user_id: Option<dal::UserPk>,
    created_at: String,
    name: String,
    policy: String,
    report: String,
    result: si_db::PolicyReportResult,
}

impl From<PolicyReport> for PolicyReportView {
    fn from(report: PolicyReport) -> Self {
        Self {
            id: report.id,
            workspace_id: report.workspace_id,
            change_set_id: report.change_set_id,
            user_id: report.user_id,
            created_at: report.created_at.to_rfc3339(),
            name: report.name,
            policy: report.policy,
            report: report.report,
            result: report.result,
        }
    }
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PolicyReportError {
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] Box<dal::TransactionsError>),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::SiDbError),
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
    Router::new()
        .route("/", get(fetch_batch::fetch_batch))
        .route("/:id", get(fetch_batch::fetch_single))
        .route("/grouped-by-name", get(grouped_by_name::grouped_by_name))
}
