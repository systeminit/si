use axum::{
    response::{IntoResponse, Response},
    routing::{delete, put},
    Router,
};
use thiserror::Error;

use crate::{service::ApiError, AppState};

mod add_individual_approver;
mod new;
mod remove;
mod remove_individual_approver;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ApprovalRequirementDefinitionError {
    #[error("dal approval requirement error: {0}")]
    DalApprovalRequirement(#[from] dal::approval_requirement::ApprovalRequirementError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] dal::TransactionsError),
}

impl IntoResponse for ApprovalRequirementDefinitionError {
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
        .route("/", put(new::new))
        .route("/:id", delete(remove::remove))
        .route(
            "/:id/individual-approver/:user-id",
            put(add_individual_approver::add_individual_approver)
                .delete(remove_individual_approver::remove_individual_approver),
        )
}
