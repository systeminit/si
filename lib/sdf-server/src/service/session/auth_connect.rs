use axum::{
    extract::{Host, OriginalUri, State},
    Json,
};
use dal::{
    DalContext, HistoryActor, KeyPair, Tenancy, User, UserPk, Workspace, WorkspacePk,
    WorkspaceSnapshotGraph,
};
use hyper::Uri;
use permissions::{ObjectType, Relation, RelationBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_data_spicedb::SpiceDbClient;
use telemetry::tracing::warn;

use super::{SessionError, SessionResult};
use crate::{
    extract::{HandlerContext, PosthogClient, RawAccessToken},
    service::session::AuthApiErrBody,
    track, AppState, WorkspacePermissions, WorkspacePermissionsMode,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthConnectRequest {
    pub code: String,
    pub on_demand_assets: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConnectResponse {
    pub user: User,
    pub workspace: Workspace,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthReconnectResponse {
    pub user: User,
    pub workspace: Workspace,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthApiUser {
    // probably dont really care about anything here but the id
    // but we may want to cache name and email? TBD...
    pub id: UserPk,
    pub nickname: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub picture_url: Option<String>,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthApiWorkspace {
    pub id: WorkspacePk,
    pub display_name: String,
    pub token: String,
    // dont need to do anything with these for now
    pub creator_user_id: UserPk,
    pub instance_url: String,
    pub instance_env_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthApiConnectResponse {
    pub user: AuthApiUser,
    pub workspace: AuthApiWorkspace,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthApiReconnectResponse {
    pub user: AuthApiUser,
    pub workspace: AuthApiWorkspace,
    pub on_demand_assets: Option<bool>,
}

// TODO: pull value from env vars / dotenv files
#[allow(clippy::too_many_arguments)]
async fn find_or_create_user_and_workspace(
    mut ctx: DalContext,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): PosthogClient,
    auth_api_user: AuthApiUser,
    auth_api_workspace: AuthApiWorkspace,
    create_workspace_permissions: WorkspacePermissionsMode,
    create_workspace_allowlist: &[String],
    on_demand_assets: bool,
    spicedb_client: Option<SpiceDbClient>,
) -> SessionResult<(User, Workspace)> {
    // lookup user or create if we've never seen it before
    let maybe_user = User::get_by_pk(&ctx, auth_api_user.id).await?;
    let user = match maybe_user {
        Some(user) => user,
        None => {
            User::new(
                &ctx,
                auth_api_user.id,
                auth_api_user.nickname,
                auth_api_user.email,
                auth_api_user.picture_url,
            )
            .await?
        }
    };
    ctx.update_history_actor(HistoryActor::User(user.pk()));

    // lookup workspace or create if we've never seen it before
    let maybe_workspace = Workspace::get_by_pk(&ctx, &auth_api_workspace.id).await?;
    let workspace = match maybe_workspace {
        Some(mut workspace) => {
            ctx.update_tenancy(Tenancy::new(*workspace.pk()));

            if workspace.token().is_none() {
                workspace.set_token(&ctx, auth_api_workspace.token).await?;
            }

            if workspace.snapshot_version() != WorkspaceSnapshotGraph::current_discriminant() {
                return Err(SessionError::WorkspaceNotYetMigrated(*workspace.pk()));
            }

            workspace
        }
        None => {
            let create_permission = user_has_permission_to_create_workspace(
                &ctx,
                &user,
                create_workspace_permissions,
                create_workspace_allowlist,
            )
            .await?;
            if !create_permission {
                warn!(
                    "user: {} has no permissions to create workspace: {:#?}",
                    &user.email(),
                    create_workspace_allowlist
                );
                return Err(SessionError::WorkspacePermissions);
            }

            let workspace = if on_demand_assets {
                Workspace::new_for_on_demand_assets(
                    &mut ctx,
                    auth_api_workspace.id,
                    auth_api_workspace.display_name.clone(),
                    auth_api_workspace.token,
                )
                .await?
            } else {
                Workspace::new_from_builtin(
                    &mut ctx,
                    auth_api_workspace.id,
                    auth_api_workspace.display_name.clone(),
                    auth_api_workspace.token,
                )
                .await?
            };

            let _key_pair = KeyPair::new(&ctx, "default").await?;

            track(
                &posthog_client,
                &ctx,
                original_uri,
                host_name,
                "workspace_first_loaded",
                serde_json::json!({
                    "change_set_id": ctx.change_set_id(),
                    "workspace_id": workspace.pk(),
                    "workspace_name": auth_api_workspace.display_name,
                    "user_id": user.pk(),
                    "on_demand_assets": on_demand_assets,
                }),
            );

            workspace
        }
    };

    if let Some(client) = spicedb_client {
        // the creator is the owner. Currently, owners cannot be changed so this should always be
        // true. Once we map the auth-api roles to spicedb we can rely on that to tell us this
        // information.
        if auth_api_workspace.creator_user_id == user.pk() {
            set_owner_as_owner(
                client,
                auth_api_user.id.to_string(),
                workspace.pk().to_string(),
            )
            .await?;
        }
    }

    // ensure workspace is associated to user
    user.associate_workspace(&ctx, *workspace.pk()).await?;

    ctx.commit_no_rebase().await?;

    Ok((user, workspace))
}

pub async fn auth_connect(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    State(state): State<AppState>,
    Json(request): Json<AuthConnectRequest>,
) -> SessionResult<Json<AuthConnectResponse>> {
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/complete-auth-connect", state.auth_api_url()))
        .json(&json!({"code": request.code }))
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

    let res_body = res.json::<AuthApiConnectResponse>().await?;

    let ctx = builder.build_default().await?;

    let (user, workspace) = find_or_create_user_and_workspace(
        ctx,
        &original_uri,
        &host_name,
        PosthogClient(posthog_client),
        res_body.user,
        res_body.workspace,
        state.create_workspace_permissions(),
        state.create_workspace_allowlist(),
        request.on_demand_assets.unwrap_or(false),
        state.spicedb_client().cloned(),
    )
    .await?;

    Ok(Json(AuthConnectResponse {
        user,
        workspace,
        token: res_body.token,
    }))
}

pub async fn auth_reconnect(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    RawAccessToken(raw_access_token): RawAccessToken,
    State(state): State<AppState>,
) -> SessionResult<Json<AuthReconnectResponse>> {
    let client = reqwest::Client::new();
    let auth_response = client
        .get(format!("{}/auth-reconnect", state.auth_api_url()))
        .bearer_auth(&raw_access_token)
        .send()
        .await?;

    if auth_response.status() != reqwest::StatusCode::OK {
        let res_err_body = auth_response
            .json::<AuthApiErrBody>()
            .await
            .map_err(|err| SessionError::AuthApiError(err.to_string()))?;
        println!("reconnect failed = {:?}", res_err_body.message);
        return Err(SessionError::AuthApiError(res_err_body.message));
    }

    let auth_response_body = auth_response.json::<AuthApiReconnectResponse>().await?;

    let ctx = builder.build_default().await?;

    let (user, workspace) = find_or_create_user_and_workspace(
        ctx,
        &original_uri,
        &host_name,
        PosthogClient(posthog_client),
        auth_response_body.user,
        auth_response_body.workspace,
        state.create_workspace_permissions(),
        state.create_workspace_allowlist(),
        auth_response_body.on_demand_assets.unwrap_or(false),
        state.spicedb_client().cloned(),
    )
    .await?;

    Ok(Json(AuthReconnectResponse { user, workspace }))
}

pub async fn user_has_permission_to_create_workspace(
    ctx: &DalContext,
    user: &User,
    mode: WorkspacePermissionsMode,
    allowlist: &[WorkspacePermissions],
) -> SessionResult<bool> {
    match mode {
        WorkspacePermissionsMode::Open => Ok(true),
        WorkspacePermissionsMode::Closed => Ok(user.is_first_user(ctx).await?),
        WorkspacePermissionsMode::Allowlist => {
            if user.is_first_user(ctx).await? {
                Ok(true)
            } else {
                let allowed = allowlist.iter().any(|entry| {
                    if entry.starts_with("*@") {
                        let mut chars = entry.chars();
                        chars.next();
                        user.email().ends_with(chars.as_str())
                    } else if entry.starts_with('@') {
                        user.email().ends_with(entry)
                    } else {
                        user.email() == entry
                    }
                });

                Ok(allowed)
            }
        }
    }
}

async fn set_owner_as_owner(
    client: SpiceDbClient,
    user_id: String,
    workspace_id: String,
) -> SessionResult<()> {
    let owner_relation = RelationBuilder::new()
        .object(ObjectType::Workspace, workspace_id.clone())
        .relation(Relation::Owner);

    // check if an owner exists already
    if owner_relation.read(client.clone()).await?.is_empty() {
        // if not, add this user as the owner
        owner_relation
            .subject(ObjectType::User, user_id)
            .create(client.clone())
            .await?;
    };
    Ok(())
}
