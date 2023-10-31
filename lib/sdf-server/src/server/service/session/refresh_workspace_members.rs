use super::{SessionError, SessionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, RawAccessToken};
use crate::service::session::AuthApiErrBody;
use axum::Json;
use dal::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefreshWorkspaceMembersRequest {
    pub workspace_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMember {
    pub user_id: String,
    pub email: String,
    pub nickname: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshWorkspaceMembersResponse {
    pub success: bool,
}

pub async fn refresh_workspace_members(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    Json(request): Json<RefreshWorkspaceMembersRequest>,
) -> SessionResult<Json<RefreshWorkspaceMembersResponse>> {
    let client = reqwest::Client::new();
    let auth_api_url = match option_env!("LOCAL_AUTH_STACK") {
        Some(_) => "http://localhost:9001",
        None => "https://auth-api.systeminit.com",
    };

    let res = client
        .get(format!(
            "{}/workspace/{}/members",
            auth_api_url,
            request.workspace_id.clone()
        ))
        .bearer_auth(&raw_access_token)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        let res_err_body = res
            .json::<AuthApiErrBody>()
            .await
            .map_err(|err| SessionError::AuthApiError(err.to_string()))?;
        println!("code exchange failed = {:?}", res_err_body.message);
        return Err(SessionError::AuthApiError(res_err_body.message));
    }

    let workspace_members = res.json::<Vec<WorkspaceMember>>().await?;

    let ctx = builder.build_head(access_builder).await?;
    let members = User::list_members_for_workspace(&ctx, request.workspace_id.clone()).await?;
    let member_ids: Vec<_> = workspace_members.into_iter().map(|w| w.user_id).collect();
    let users_to_remove: Vec<_> = members
        .into_iter()
        .filter(|u| !member_ids.contains(&u.pk().to_string()))
        .collect();

    for remove in users_to_remove {
        println!("Removing User: {}", remove.pk().clone());
        User::delete_user_from_workspace(&ctx, remove.pk(), request.workspace_id.clone()).await?;
    }

    Ok(Json(RefreshWorkspaceMembersResponse { success: true }))
}
