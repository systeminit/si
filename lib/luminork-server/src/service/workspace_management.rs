use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{
        delete,
        get,
        patch,
        post,
    },
};
use chrono::{
    DateTime,
    Utc,
};
use create_workspace::CreateWorkspaceRequest;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::WorkspaceId;
use thiserror::Error;
use update_workspace::UpdateWorkspaceRequest;
use utoipa::{
    OpenApi,
    ToSchema,
};

use crate::{
    AppState,
    extract::workspace::{
        AuthorizedForWorkspaceManagement,
        TargetWorkspaceIdFromPath,
    },
};

mod create_workspace;
mod delete_workspace;
mod get_workspace;
mod list_workspaces;
mod update_workspace;

pub type WorkspaceManagementResult<T> = Result<T, WorkspaceManagementError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceManagementError {
    #[error("auth api error: {message}")]
    AuthApiError { status: StatusCode, message: String },
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

impl IntoResponse for WorkspaceManagementError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl From<JsonRejection> for WorkspaceManagementError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => WorkspaceManagementError::Validation(format!(
                "Invalid JSON data format: {rejection}"
            )),
            JsonRejection::JsonSyntaxError(_) => {
                WorkspaceManagementError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => WorkspaceManagementError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => {
                WorkspaceManagementError::Validation(format!("JSON validation error: {rejection}"))
            }
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for WorkspaceManagementError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            WorkspaceManagementError::AuthApiError { status, message } => {
                (*status, message.clone())
            }
            WorkspaceManagementError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            WorkspaceManagementError::Request(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthApiErrBody {
    pub kind: String,
    pub message: String,
}

// Helper function to handle auth-api error responses
async fn handle_auth_api_error(res: reqwest::Response) -> WorkspaceManagementError {
    let status = res.status();
    let status_code = match status.as_u16() {
        400 => StatusCode::BAD_REQUEST,
        401 => StatusCode::UNAUTHORIZED,
        403 => StatusCode::FORBIDDEN,
        404 => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let message = match res.json::<AuthApiErrBody>().await {
        Ok(err_body) => err_body.message,
        Err(_) => format!("Auth API returned error status: {status}"),
    };

    WorkspaceManagementError::AuthApiError {
        status: status_code,
        message,
    }
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest(
            "/workspaces",
            Router::new()
                .route("/", get(list_workspaces::list_workspaces))
                .route("/", post(create_workspace::create_workspace))
                .nest(
                    "/:workspace_id",
                    Router::new()
                        .route("/", get(get_workspace::get_workspace))
                        .route("/", delete(delete_workspace::delete_workspace))
                        .route("/", patch(update_workspace::update_workspace))
                        .layer(middleware::from_extractor::<TargetWorkspaceIdFromPath>()), //
                                                                                           // ,
                                                                                           // .route("/members", get(get_workspace_members::get_workspace_members)),
                                                                                           // .route("/members", delete(delete_workspace_member::delete_workspace_member)),
                                                                                           // .route("/members", post(create_workspace_member::create_workspace_member)),
                                                                                           // .route("/update_member_access", post(update_member_access::update_member_access)), <---- REQUIRES TALKING TO DAL!!
                ),
        )
        .route_layer(middleware::from_extractor_with_state::<
            AuthorizedForWorkspaceManagement,
            AppState,
        >(state))
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "System Initiative API - Workspace Management",
        description = "System Initiative External API server - Workspace Management Routes",
        version = "1.0.0"
    ),
    servers(
        (url = "/management", description = "Workspace Management API")
    ),
    paths(
        list_workspaces::list_workspaces,
        get_workspace::get_workspace,
        delete_workspace::delete_workspace,
        update_workspace::update_workspace,
        create_workspace::create_workspace,
    ),
    components(schemas(
        Workspace,
        WorkspaceManagementRequestPath,
        CreatorUser,
        CreateWorkspaceRequest,
        UpdateWorkspaceRequest,
    )),
    tags()
)]
pub struct WorkspaceManagementApiDoc;

pub fn get_openapi() -> utoipa::openapi::OpenApi {
    WorkspaceManagementApiDoc::openapi()
}

#[derive(Deserialize, ToSchema)]
pub struct WorkspaceManagementRequestPath {
    #[schema(value_type = String)]
    pub workspace_id: WorkspaceId,
}

// Auth API Response types
// These are only used for internal translation to public facing data
// For example, we shouldn't return the secret key for a workspace
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthApiWorkspace {
    pub id: String,
    pub instance_env_type: String,
    pub instance_url: Option<String>,
    pub display_name: String,
    pub creator_user_id: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub token: Option<String>,
    pub is_default: bool,
    pub quarantined_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub is_favourite: bool,
    pub is_hidden: bool,
    pub approvals_enabled: bool,
    #[serde(default)]
    pub role: Option<String>,
    pub invited_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub creator_user: Option<CreatorUser>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthApiCreateWorkspaceResponse {
    pub workspaces: Vec<AuthApiWorkspace>,
    pub new_workspace_id: String,
}

// Public workspace response object
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    #[schema(value_type = String)]
    pub id: String,
    #[schema(value_type = String)]
    pub instance_env_type: String,
    #[schema(value_type = Option<String>)]
    pub instance_url: Option<String>,
    #[schema(value_type = String)]
    pub display_name: String,
    #[schema(value_type = String)]
    pub creator_user_id: String,
    #[schema(value_type = bool)]
    pub is_default: bool,
    #[schema(value_type = Option<String>)]
    pub quarantined_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>)]
    pub description: Option<String>,
    #[schema(value_type = bool)]
    pub approvals_enabled: bool,
    #[schema(value_type = Option<String>)]
    pub role: Option<String>,
    #[schema(value_type = Option<CreatorUser>)]
    pub creator_user: Option<CreatorUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatorUser {
    #[schema(value_type = Option<String>)]
    pub first_name: Option<String>,
    #[schema(value_type = Option<String>)]
    pub last_name: Option<String>,
}

impl From<AuthApiWorkspace> for Workspace {
    fn from(auth: AuthApiWorkspace) -> Self {
        Workspace {
            id: auth.id,
            instance_env_type: auth.instance_env_type,
            instance_url: auth.instance_url,
            display_name: auth.display_name,
            creator_user_id: auth.creator_user_id,
            is_default: auth.is_default,
            quarantined_at: auth.quarantined_at,
            description: auth.description,
            approvals_enabled: auth.approvals_enabled,
            role: auth.role,
            creator_user: auth.creator_user,
        }
    }
}
