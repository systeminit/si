use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        State,
    },
};
use dal::{
    DalContext,
    KeyPair,
    UserPk,
    Workspace,
    WorkspacePk,
    WorkspaceSnapshotGraph,
    workspace::SnapshotVersion,
    workspace_integrations::WorkspaceIntegration,
    workspace_snapshot::split_snapshot::SuperGraphVersionDiscriminants,
};
use hyper::Uri;
use permissions::{
    Relation,
    RelationBuilder,
};
use sdf_core::{
    app_state::AppState,
    tracking::track,
    workspace_permissions::{
        WorkspacePermissions,
        WorkspacePermissionsMode,
    },
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    request::{
        RawAccessToken,
        RequestUlidFromHeader,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_data_spicedb::SpiceDbClient;
use si_db::{
    HistoryActor,
    Tenancy,
    User,
};
use si_events::audit_log::AuditLogKind;
use telemetry::tracing::warn;

use crate::{
    AuthApiErrBody,
    SessionError,
    SessionResult,
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
    pub user_workspace_flags: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthReconnectResponse {
    pub user: User,
    pub workspace: Workspace,
    pub user_workspace_flags: serde_json::Value,
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
    spicedb_client: Option<&mut SpiceDbClient>,
) -> SessionResult<(DalContext, User, Workspace)> {
    // lookup user or create if we've never seen it before
    let maybe_user = User::get_by_pk_opt(&ctx, auth_api_user.id).await?;
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
    let maybe_workspace = Workspace::get_by_pk_opt(&ctx, auth_api_workspace.id).await?;
    let workspace = match maybe_workspace {
        Some(mut workspace) => {
            ctx.update_tenancy(Tenancy::new(*workspace.pk()));

            if workspace.token().is_none() {
                workspace.set_token(&ctx, auth_api_workspace.token).await?;
            }

            if workspace.snapshot_version()
                != SnapshotVersion::Legacy(WorkspaceSnapshotGraph::current_discriminant())
                && workspace.snapshot_version()
                    != SnapshotVersion::Split(SuperGraphVersionDiscriminants::current_discriminant())
            {
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
                return Err(SessionError::WorkspacePermission(
                    "you do not have permission to create a workspace on this instance",
                ));
            }

            let workspace = Workspace::new_for_on_demand_assets(
                &mut ctx,
                auth_api_workspace.id,
                auth_api_workspace.display_name.clone(),
                auth_api_workspace.token,
            )
            .await?;

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
                    "on_demand_assets": true,
                }),
            );

            workspace
        }
    };

    if let Some(client) = spicedb_client {
        // the creator is the owner. Currently, owners cannot be changed so this should always be
        // true. Once we map the auth-api roles to spicedb we can rely on that to tell us this
        // information.
        ensure_workspace_creator_is_owner_of_workspace(
            client,
            auth_api_workspace.creator_user_id,
            *workspace.pk(),
        )
        .await?;
    }

    // ensure workspace is associated to user
    user.associate_workspace(&ctx, *workspace.pk()).await?;

    // Check the workspace integration
    // We need to ensure that the integrations row is available for older workspaces
    if WorkspaceIntegration::get_integrations_for_workspace_pk(&ctx)
        .await?
        .is_none()
    {
        WorkspaceIntegration::new(&ctx, None).await?;
    }

    ctx.write_audit_log_to_head(AuditLogKind::Login {}, "Person".to_string())
        .await?;

    ctx.commit_no_rebase().await?;

    Ok((ctx, user, workspace))
}

pub async fn auth_connect(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    RequestUlidFromHeader(request_ulid): RequestUlidFromHeader,
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

    let ctx = builder.build_default(request_ulid).await?;

    let (ctx, user, workspace) = find_or_create_user_and_workspace(
        ctx,
        &original_uri,
        &host_name,
        PosthogClient(posthog_client),
        res_body.user,
        res_body.workspace,
        state.create_workspace_permissions(),
        state.create_workspace_allowlist(),
        state.spicedb_client_clone().as_mut(),
    )
    .await?;

    let user_workspace_flags =
        User::get_flags_for_user_on_workspace(&ctx, user.pk(), *workspace.pk()).await?;

    Ok(Json(AuthConnectResponse {
        user,
        workspace,
        token: res_body.token,
        user_workspace_flags,
    }))
}

pub async fn auth_reconnect(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    RequestUlidFromHeader(request_ulid): RequestUlidFromHeader,
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

    let ctx = builder.build_default(request_ulid).await?;

    let (ctx, user, workspace) = find_or_create_user_and_workspace(
        ctx,
        &original_uri,
        &host_name,
        PosthogClient(posthog_client),
        auth_response_body.user,
        auth_response_body.workspace,
        state.create_workspace_permissions(),
        state.create_workspace_allowlist(),
        state.spicedb_client_clone().as_mut(),
    )
    .await?;

    let user_workspace_flags =
        User::get_flags_for_user_on_workspace(&ctx, user.pk(), *workspace.pk()).await?;

    Ok(Json(AuthReconnectResponse {
        user,
        workspace,
        user_workspace_flags,
    }))
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

async fn ensure_workspace_creator_is_owner_of_workspace(
    client: &mut SpiceDbClient,
    user_id: UserPk,
    workspace_id: WorkspacePk,
) -> SessionResult<()> {
    let owner_relation = RelationBuilder::new()
        .workspace_object(workspace_id)
        .relation(Relation::Owner);

    // Cut down on the amount of `String` allocations dealing with ids
    let mut user_id_buf = UserPk::array_to_str_buf();
    let user_id_str = user_id.array_to_str(&mut user_id_buf);

    // check if an owner relation exists for the user already
    let is_user_an_owner = owner_relation
        .read(client)
        .await?
        .iter()
        .any(|rel| rel.subject().id() == user_id_str);

    if !is_user_an_owner {
        // if not, create the relation
        owner_relation.user_subject(user_id).create(client).await?;
    };
    Ok(())
}
