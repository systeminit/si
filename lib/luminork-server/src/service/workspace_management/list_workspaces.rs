use axum::{
    Json,
    extract::State,
};
use sdf_extract::request::RawAccessToken;

use super::{
    AuthApiWorkspace,
    Workspace,
    WorkspaceManagementResult,
    handle_auth_api_error,
};
use crate::{
    AppState,
    extract::PosthogEventTracker,
};

#[utoipa::path(
    get,
    path = "/management/workspaces",
    tag = "workspace_management",
    summary = "List workspaces",
    responses(
        (status = 200, description = "Workspaces Listed successfully", body = Vec<Workspace>),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_workspaces(
    RawAccessToken(token): RawAccessToken,
    State(state): State<AppState>,
    _tracker: PosthogEventTracker,
) -> WorkspaceManagementResult<Json<Vec<Workspace>>> {
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/workspaces", state.auth_api_url()))
        .bearer_auth(token)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(handle_auth_api_error(res).await);
    }

    let auth_workspaces = res.json::<Vec<AuthApiWorkspace>>().await?;

    // Filter out deleted workspaces
    let workspaces: Vec<Workspace> = auth_workspaces
        .into_iter()
        .filter(|w| w.deleted_at.is_none())
        .map(|w| w.into())
        .collect();

    Ok(Json(workspaces))
}
