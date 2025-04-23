use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        State,
    },
    http::uri::Uri,
};
use dal::{
    DalContext,
    User,
};
use permissions::{
    ObjectType,
    Relation,
    RelationBuilder,
};
use sdf_core::{
    app_state::AppState,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    request::RawAccessToken,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_spicedb::SpiceDbClient;
use strum::{
    Display,
    EnumString,
};

use crate::{
    AuthApiErrBody,
    SessionError,
    SessionResult,
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

#[allow(clippy::too_many_arguments)]
pub async fn refresh_workspace_members(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    State(mut state): State<AppState>,
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
    let ctx = builder.build_head(access_builder).await?;
    let posthog_client = PosthogClient(posthog_client.clone());

    if let Some(client) = state.spicedb_client() {
        let approvers: Vec<_> = workspace_members
            .clone()
            .into_iter()
            .filter(|u| matches!(u.role, WorkspaceRole::Approver))
            .map(|u| u.user_id)
            .collect();
        sync_workspace_approvers(
            &ctx,
            client,
            request.workspace_id.clone(),
            approvers,
            &original_uri,
            &host_name,
            &posthog_client,
        )
        .await?;
    }

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
    ctx: &DalContext,
    client: &mut SpiceDbClient,
    workspace_id: String,
    new_approver_ids: Vec<String>,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): &PosthogClient,
) -> SessionResult<()> {
    let existing_approvers = RelationBuilder::new()
        .object(ObjectType::Workspace, workspace_id.clone())
        .relation(Relation::Approver)
        .read(client)
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
        .clone()
        .into_iter()
        .filter(|r| !new_approver_ids.contains(r))
        .collect();

    track(
        posthog_client,
        ctx,
        original_uri,
        host_name,
        "sync_workspace_approvers",
        serde_json::json!({
            "how": "/session/refresh_workspace_members",
            "to_add": to_add.clone(),
            "to_remove": to_remove.clone(),
            "existing_approver_ids": existing_approver_ids,
        }),
    );

    for user_pk_str in to_add {
        RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_id.clone())
            .relation(Relation::Approver)
            .subject(ObjectType::User, user_pk_str.clone())
            .create(client)
            .await?;

        track(
            posthog_client,
            ctx,
            original_uri,
            host_name,
            "add_approver",
            serde_json::json!({
                "how": "/session/refresh_workspace_member",
                "user_pk": user_pk_str,
            }),
        );
    }

    for user_pk_str in to_remove {
        RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_id.clone())
            .relation(Relation::Approver)
            .subject(ObjectType::User, user_pk_str.clone())
            .delete(client)
            .await?;

        track(
            posthog_client,
            ctx,
            original_uri,
            host_name,
            "remove_approver",
            serde_json::json!({
                "how": "/session/refresh_workspace_member",
                "user_pk": user_pk_str,
            }),
        );
    }
    Ok(())
}
