use std::str::FromStr;

use axum::{
    Json,
    extract::{
        Host,
        State,
    },
};
use chrono::{
    DateTime,
    Utc,
};
use dal::{
    DalContextBuilder,
    KeyPair,
    Workspace as DalWorkspace,
    workspace_integrations::WorkspaceIntegration,
};
use permissions::{
    ObjectType,
    Relation,
    RelationBuilder,
};
use sdf_extract::request::{
    RawAccessToken,
    RequestUlidFromHeader,
    ValidatedToken,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    HistoryActor,
    Tenancy,
    User,
};
use si_events::WorkspacePk;
use si_id::WorkspaceId;
use utoipa::ToSchema;

use super::{
    AuthApiCreateWorkspaceResponse,
    InitialApiToken,
    Workspace,
    WorkspaceManagementError,
    WorkspaceManagementResult,
    handle_auth_api_error,
};
use crate::{
    AppState,
    extract::{
        HandlerContext,
        PosthogEventTracker,
    },
};

#[utoipa::path(
    post,
    path = "/management/workspaces",
    request_body = CreateWorkspaceRequest,
    tag = "workspace_management",
    summary = "Create a new workspace",
    responses(
        (status = 200, description = "Workspace successfully created", body = Workspace),
        (status = 400, description = "Bad Request - Validation error (invalid URL, display name, or description format)"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn create_workspace(
    HandlerContext(builder): HandlerContext,
    validated_token: ValidatedToken,
    RawAccessToken(token): RawAccessToken,
    RequestUlidFromHeader(request_ulid): RequestUlidFromHeader,
    State(state): State<AppState>,
    Host(host_name): Host,
    tracker: PosthogEventTracker,
    payload: Result<Json<CreateWorkspaceRequest>, axum::extract::rejection::JsonRejection>,
) -> WorkspaceManagementResult<Json<Workspace>> {
    let Json(payload) = payload?;

    // Validate that the instance URL matches the API FQDN
    validate_instance_url(&host_name, &payload.instance_url)?;

    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/workspaces/new", state.auth_api_url()))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(handle_auth_api_error(res).await);
    }

    let auth_response = res.json::<AuthApiCreateWorkspaceResponse>().await?;
    let new_workspace = auth_response
        .workspaces
        .into_iter()
        .find(|w| w.id == auth_response.new_workspace_id)
        .ok_or_else(|| WorkspaceManagementError::AuthApiError {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message: "Created workspace not found in response".to_string(),
        })?;

    let auth_token_request = CreateAuthTokenRequest {
        name: "initial-workspace-token".to_string(),
        expiration: "1y".to_string(),
    };

    let auth_token_response = client
        .post(format!(
            "{}/workspaces/{}/authTokens",
            state.auth_api_url(),
            new_workspace.id
        ))
        .bearer_auth(&token)
        .json(&auth_token_request)
        .send()
        .await?;

    if auth_token_response.status() != reqwest::StatusCode::OK {
        return Err(handle_auth_api_error(auth_token_response).await);
    }

    let auth_token_data = auth_token_response
        .json::<AuthApiCreateAuthTokenResponse>()
        .await?;

    // Sync user and workspace to DAL
    sync_user_and_workspace(
        &builder,
        &state,
        &WorkspaceId::from_str(&new_workspace.id).map_err(|e| {
            WorkspaceManagementError::Validation(format!("Invalid workspace_id: {e}"))
        })?,
        &validated_token,
        request_ulid,
        &new_workspace,
        &tracker,
    )
    .await?;

    // Build the workspace response with the initial API token
    let mut workspace: Workspace = new_workspace.into();
    workspace.initial_api_token = Some(InitialApiToken {
        token: auth_token_data.token,
        expires_at: auth_token_data.auth_token.expires_at,
    });

    Ok(Json(workspace))
}

/// Syncs user and workspace from auth-api to the DAL and SpiceDB
/// This is called after workspace creation to ensure the workspace exists in our database
async fn sync_user_and_workspace(
    builder: &DalContextBuilder,
    state: &AppState,
    workspace_id: &WorkspaceId,
    validated_token: &ValidatedToken,
    request_ulid: Option<ulid::Ulid>,
    auth_workspace: &super::AuthApiWorkspace,
    tracker: &PosthogEventTracker,
) -> WorkspaceManagementResult<()> {
    let workspace_pk = WorkspacePk::from_str(&workspace_id.to_string())
        .map_err(|e| WorkspaceManagementError::Validation(format!("Invalid workspace_id: {e}")))?;

    let user_id = validated_token.0.custom.user_id();

    // Build a default context first (workspace doesn't exist yet in DAL)
    let mut ctx = builder.build_default(request_ulid).await?;

    let user = User::get_by_pk_opt(&ctx, user_id)
        .await?
        .ok_or_else(|| {
            WorkspaceManagementError::UserNotFound(format!(
                "User {user_id} not found in database. Users must be created during authentication before creating workspaces."
            ))
        })?;
    ctx.update_history_actor(HistoryActor::User(user.pk()));

    // Create new workspace in DAL (Auth API has already created it)
    let token_str = auth_workspace.token.as_deref().unwrap_or("");
    let workspace = DalWorkspace::new_for_on_demand_assets(
        &mut ctx,
        workspace_pk,
        &auth_workspace.display_name,
        token_str,
    )
    .await
    .map_err(|e| WorkspaceManagementError::Dal(Box::new(e.into())))?;

    // Update context tenancy now that workspace exists
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));

    KeyPair::new(&ctx, "default").await?;

    tracker.track(
        &ctx,
        "api_workspace_created",
        serde_json::json!({
            "workspace_id": workspace.pk(),
            "workspace_name": auth_workspace.display_name,
            "user_id": user.pk(),
        }),
    );

    // Set up workspace creator as owner in SpiceDB
    if let Some(mut client) = state.spicedb_client_clone() {
        RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_pk.to_string())
            .relation(Relation::Owner)
            .subject(ObjectType::User, auth_workspace.creator_user_id.clone())
            .create(&mut client)
            .await?;
    }

    // Associate user with workspace
    user.associate_workspace(&ctx, *workspace.pk()).await?;

    // Ensure workspace integration exists
    if WorkspaceIntegration::get_integrations_for_workspace_pk(&ctx)
        .await?
        .is_none()
    {
        WorkspaceIntegration::new(&ctx, None).await?;
    }

    ctx.commit().await?;

    Ok(())
}

/// TODO(PAUL): This is a temporary measure until we can understand what environment that the request is managing
/// The better fix here is to thread through a parameter that suggests what the system is allowed to manage
/// Validates that the instance URL is appropriate for the API FQDN being called
fn validate_instance_url(api_fqdn: &str, instance_url: &str) -> WorkspaceManagementResult<()> {
    // Parse the instance URL to extract the hostname
    let instance_host = instance_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or(instance_url);

    // Remove port if present (for localhost)
    let instance_host = instance_host.split(':').next().unwrap_or(instance_host);

    // Check based on API FQDN
    let is_valid = if api_fqdn == "api.systeminit.com" || api_fqdn == "api.systeminit.com:443" {
        // Production API can only create workspaces for app.systeminit.com
        instance_host == "app.systeminit.com"
    } else if api_fqdn == "api.tools.systeminit.com" || api_fqdn == "api.tools.systeminit.com:443" {
        // Tools API can only create workspaces for tools.systeminit.com
        instance_host == "tools.systeminit.com"
    } else if api_fqdn.starts_with("localhost") || api_fqdn.starts_with("127.0.0.1") {
        // Localhost can create workspaces for localhost, tools, or app
        instance_host == "localhost"
            || instance_host == "127.0.0.1"
            || instance_host == "tools.systeminit.com"
            || instance_host == "app.systeminit.com"
    } else {
        // Unknown API FQDN - reject
        false
    };

    if !is_valid {
        return Err(WorkspaceManagementError::InvalidInstanceUrl(format!(
            "Instance URL '{instance_url}' is not valid for API endpoint '{api_fqdn}'. Expected instance URL based on API endpoint."
        )));
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    #[schema(example = "https://app.systeminit.com")]
    pub instance_url: String,
    #[schema(example = "My Production Workspace")]
    pub display_name: String,
    #[schema(example = "Production environment for customer deployments")]
    pub description: String,
    #[serde(default)]
    #[schema(example = false, default = false)]
    pub is_default: bool,
}

// Auth API authTokens request type
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAuthTokenRequest {
    pub name: String,
    pub expiration: String,
}

// Auth API authTokens response types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthApiCreateAuthTokenResponse {
    pub auth_token: AuthApiAuthToken,
    pub token: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthApiAuthToken {
    pub id: String,
    pub name: Option<String>,
    pub user_id: String,
    pub workspace_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub claims: serde_json::Value,
    pub last_used_at: Option<DateTime<Utc>>,
    pub last_used_ip: Option<String>,
}
