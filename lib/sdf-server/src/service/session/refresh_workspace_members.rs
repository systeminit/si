use axum::{extract::State, Json};
use dal::User;
use permissions::{ObjectType, Relation, RelationBuilder};
use serde::{Deserialize, Serialize};
use si_data_spicedb::SpiceDbClient;
use strum::{Display, EnumString};

use super::{SessionError, SessionResult};
use crate::{
    extract::{AccessBuilder, HandlerContext, RawAccessToken},
    service::session::AuthApiErrBody,
    AppState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefreshWorkspaceMembersRequest {
    pub workspace_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMember {
    pub user_id: String,
    pub email: String,
    pub nickname: String,
    pub role: WorkspaceRole,
}

#[derive(Clone, Display, Debug, Deserialize, EnumString, Serialize)]
#[strum(serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum WorkspaceRole {
    Approver,
    Editor,
    Owner,
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
    State(state): State<AppState>,
    Json(request): Json<RefreshWorkspaceMembersRequest>,
) -> SessionResult<Json<RefreshWorkspaceMembersResponse>> {
    let client = reqwest::Client::new();

    let res = client
        .get(format!(
            "{}/workspace/{}/members",
            state.auth_api_url(),
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

    if let Some(client) = state.spicedb_client() {
        let approvers: Vec<_> = workspace_members
            .clone()
            .into_iter()
            .filter(|u| matches!(u.role, WorkspaceRole::Approver))
            .map(|u| u.user_id)
            .collect();
        sync_workspace_approvers(client.clone(), request.workspace_id.clone(), approvers).await?;
    }

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

async fn sync_workspace_approvers(
    client: SpiceDbClient,
    workspace_id: String,
    new_approver_ids: Vec<String>,
) -> SessionResult<()> {
    let existing_approvers = RelationBuilder::new()
        .object(ObjectType::Workspace, workspace_id.clone())
        .relation(Relation::Approver)
        .read(client.clone())
        .await?;

    let existing_approver_ids: Vec<_> = existing_approvers
        .into_iter()
        .map(|w| w.subject().id().to_string())
        .collect();

    let to_add: Vec<_> = new_approver_ids
        .clone()
        .into_iter()
        .filter(|u| !existing_approver_ids.contains(u))
        .collect();

    let to_remove: Vec<_> = existing_approver_ids
        .into_iter()
        .filter(|r| !new_approver_ids.contains(r))
        .collect();

    for user in to_add {
        RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_id.clone())
            .relation(Relation::Approver)
            .subject(ObjectType::User, user)
            .create(client.clone())
            .await?;
    }

    for user in to_remove {
        RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_id.clone())
            .relation(Relation::Approver)
            .subject(ObjectType::User, user)
            .delete(client.clone())
            .await?;
    }
    Ok(())
}
