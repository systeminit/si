use axum::{
    Router,
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
    UserPk,
    WorkspacePk,
    workspace_integrations::WorkspaceIntegration,
};
use hyper::StatusCode;
use sdf_core::api_error::ApiError;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::AppState;

pub mod get_integrations;
pub mod update_integration;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum IntegrationsError {
    #[error("integration for workspace {0} not found")]
    IntegrationNotFoundForWorkspace(WorkspacePk),
    #[error("invalid user found")]
    InvalidUser,
    #[error("permissions error: {0}")]
    Permissions(#[from] permissions::Error),
    #[error("SpiceDb client not found")]
    SpiceDbClientNotFound,
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("user unable to approve integration: {0}")]
    UserUnableToApproveIntegration(UserPk),
    #[error("workspace integration error: {0}")]
    WorkspaceIntegrations(#[from] dal::workspace_integrations::WorkspaceIntegrationsError),
}

pub type IntegrationsResult<T> = Result<T, IntegrationsError>;

impl IntoResponse for IntegrationsError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(update_integration::update_integration))
        .route("/", get(get_integrations::get_integration))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationResponse {
    pub workspace_pk: WorkspacePk,
    pub slack_webhook_url: Option<String>,
}

impl From<WorkspaceIntegration> for IntegrationResponse {
    fn from(integration: WorkspaceIntegration) -> Self {
        Self {
            workspace_pk: integration.workspace_pk(),
            slack_webhook_url: integration.slack_webhook_url(),
        }
    }
}
