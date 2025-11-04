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
mod invite_member;
mod list_members;
mod list_workspaces;
mod remove_member;
mod sync_members;
mod update_member_role;
mod update_workspace;

use sync_members::sync_members;

pub type WorkspaceManagementResult<T> = Result<T, WorkspaceManagementError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceManagementError {
    #[error("auth api error: {message}")]
    AuthApiError { status: StatusCode, message: String },
    #[error("dal error: {0}")]
    Dal(#[from] Box<dal::TransactionsError>),
    #[error("invalid instance url: {0}")]
    InvalidInstanceUrl(String),
    #[error("key pair error: {0}")]
    KeyPair(#[from] Box<dal::KeyPairError>),
    #[error("permissions error: {0}")]
    Permissions(#[from] Box<permissions::Error>),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("si-db error: {0}")]
    SiDb(#[from] Box<si_db::Error>),
    #[error("user not found: {0}")]
    UserNotFound(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("workspace integration error: {0}")]
    WorkspaceIntegration(#[from] Box<dal::workspace_integrations::WorkspaceIntegrationsError>),
    #[error("workspace permission denied: {0}")]
    WorkspacePermission(String),
}

impl From<dal::TransactionsError> for WorkspaceManagementError {
    fn from(value: dal::TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<permissions::Error> for WorkspaceManagementError {
    fn from(value: permissions::Error) -> Self {
        Box::new(value).into()
    }
}

impl From<si_db::Error> for WorkspaceManagementError {
    fn from(value: si_db::Error) -> Self {
        Box::new(value).into()
    }
}

impl From<dal::workspace_integrations::WorkspaceIntegrationsError> for WorkspaceManagementError {
    fn from(value: dal::workspace_integrations::WorkspaceIntegrationsError) -> Self {
        Box::new(value).into()
    }
}

impl From<dal::KeyPairError> for WorkspaceManagementError {
    fn from(value: dal::KeyPairError) -> Self {
        Box::new(value).into()
    }
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
            WorkspaceManagementError::InvalidInstanceUrl(msg) => {
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            WorkspaceManagementError::UserNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            WorkspaceManagementError::WorkspacePermission(msg) => {
                (StatusCode::FORBIDDEN, msg.clone())
            }
            WorkspaceManagementError::Request(_)
            | WorkspaceManagementError::Dal(_)
            | WorkspaceManagementError::KeyPair(_)
            | WorkspaceManagementError::Permissions(_)
            | WorkspaceManagementError::SiDb(_)
            | WorkspaceManagementError::WorkspaceIntegration(_) => {
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
        409 => StatusCode::CONFLICT,
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
                        .route("/members", get(list_members::list_members))
                        .route("/members", post(invite_member::invite_member))
                        .route("/members", delete(remove_member::remove_member))
                        .route(
                            "/update_member_access",
                            post(update_member_role::update_member_role),
                        )
                        .layer(middleware::from_extractor::<TargetWorkspaceIdFromPath>()),
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
        list_members::list_members,
        update_member_role::update_member_role,
        invite_member::invite_member,
        remove_member::remove_member,
    ),
    components(schemas(
        Workspace,
        WorkspaceManagementRequestPath,
        CreatorUser,
        InitialApiToken,
        CreateWorkspaceRequest,
        UpdateWorkspaceRequest,
        Member,
        UpdateMemberRoleRequest,
        InviteMemberRequest,
        RemoveMemberRequest,
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
    #[schema(value_type = Option<String>)]
    pub external_id: Option<String>,
    #[schema(value_type = Option<InitialApiToken>)]
    pub initial_api_token: Option<InitialApiToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatorUser {
    #[schema(value_type = Option<String>)]
    pub first_name: Option<String>,
    #[schema(value_type = Option<String>)]
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitialApiToken {
    #[schema(value_type = String, example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    #[schema(value_type = Option<String>)]
    pub expires_at: Option<DateTime<Utc>>,
}

// Member types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    #[schema(value_type = String, example = "01GW0KXH4YJBWC7BTBAZ6ZR7EA")]
    pub user_id: String,

    #[schema(example = "user@example.com")]
    pub email: String,

    #[schema(example = "John Doe")]
    pub nickname: String,

    #[schema(example = "OWNER")]
    pub role: String,

    #[schema(value_type = Option<String>)]
    pub signup_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberRoleRequest {
    #[schema(example = "01GW0KXH4YJBWC7BTBAZ6ZR7EA")]
    pub user_id: String,

    #[schema(example = "EDITOR")]
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InviteMemberRequest {
    #[schema(example = "newuser@example.com")]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMemberRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
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
            external_id: auth.token,
            initial_api_token: None,
        }
    }
}

// Map Auth API roles (EDITOR) to Luminork roles (COLLABORATOR) for responses
fn map_member_role_from_auth_api(role: String) -> String {
    if role.eq_ignore_ascii_case("EDITOR") {
        "COLLABORATOR".to_string()
    } else {
        role
    }
}

// Map Luminork roles (COLLABORATOR) to Auth API roles (EDITOR) for requests
pub(super) fn map_role_to_auth_api(role: &str) -> String {
    if role.eq_ignore_ascii_case("COLLABORATOR") {
        "EDITOR".to_string()
    } else {
        role.to_string()
    }
}

pub(super) fn transform_members(members: Vec<Member>) -> Vec<Member> {
    members
        .into_iter()
        .map(|mut member| {
            member.role = map_member_role_from_auth_api(member.role);
            member
        })
        .collect()
}
