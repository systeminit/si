use ::serde::{
    Deserialize,
    Serialize,
};
use axum::{
    Json,
    extract::{
        Path,
        State,
    },
};
use sdf_extract::request::RawAccessToken;
use utoipa::ToSchema;

use super::{
    AuthApiWorkspace,
    Workspace,
    WorkspaceManagementError,
    WorkspaceManagementRequestPath,
    WorkspaceManagementResult,
    handle_auth_api_error,
};
use crate::{
    AppState,
    extract::PosthogEventTracker,
};

#[utoipa::path(
    patch,
    path = "/management/workspaces/{workspace_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    request_body = UpdateWorkspaceRequest,
    tag = "workspace_management",
    summary = "Update the details of a workspace",
    responses(
        (status = 200, description = "Workspace successfully updated", body = Workspace),
        (status = 400, description = "Bad Request - Validation error (invalid URL, display name, or description format)"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User must be workspace owner to update workspace"),
        (status = 404, description = "Workspace not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn update_workspace(
    RawAccessToken(token): RawAccessToken,
    State(state): State<AppState>,
    _tracker: PosthogEventTracker,
    Path(WorkspaceManagementRequestPath { workspace_id }): Path<WorkspaceManagementRequestPath>,
    payload: Result<Json<UpdateWorkspaceRequest>, axum::extract::rejection::JsonRejection>,
) -> WorkspaceManagementResult<Json<Workspace>> {
    let Json(payload) = payload?;

    let client = reqwest::Client::new();

    let get_res = client
        .get(format!(
            "{}/workspaces/{}",
            state.auth_api_url(),
            workspace_id
        ))
        .bearer_auth(&token)
        .send()
        .await?;

    if get_res.status() != reqwest::StatusCode::OK {
        return Err(handle_auth_api_error(get_res).await);
    }

    let current_workspace = get_res.json::<AuthApiWorkspace>().await?;

    let instance_url = payload
        .instance_url
        .or(current_workspace.instance_url)
        .ok_or_else(|| {
            WorkspaceManagementError::Validation(
                "instance_url is required (not set in workspace and not provided in request)"
                    .to_string(),
            )
        })?;

    let display_name = payload
        .display_name
        .unwrap_or(current_workspace.display_name);

    let description = payload
        .description
        .or(current_workspace.description)
        .ok_or_else(|| {
            WorkspaceManagementError::Validation(
                "description is required (not set in workspace and not provided in request)"
                    .to_string(),
            )
        })?;

    let update_request = AuthApiUpdateWorkspaceRequest {
        instance_url,
        display_name,
        description,
    };

    let res = client
        .patch(format!(
            "{}/workspaces/{}",
            state.auth_api_url(),
            workspace_id
        ))
        .bearer_auth(token)
        .json(&update_request)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(handle_auth_api_error(res).await);
    }

    let auth_workspaces = res.json::<Vec<AuthApiWorkspace>>().await?;
    let updated_workspace = auth_workspaces
        .into_iter()
        .find(|w| w.id == workspace_id.to_string())
        .ok_or_else(|| WorkspaceManagementError::AuthApiError {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message: "Updated workspace not found in response".to_string(),
        })?;

    Ok(Json(updated_workspace.into()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "https://app.systeminit.com")]
    pub instance_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Updated Workspace Name")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Updated description for the workspace")]
    pub description: Option<String>,
}

// Internal struct for sending to auth-api (requires all fields)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthApiUpdateWorkspaceRequest {
    pub instance_url: String,
    pub display_name: String,
    pub description: String,
}
