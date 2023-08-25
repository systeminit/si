use super::{SessionError, SessionResult};
use crate::server::extract::{HandlerContext, RawAccessToken};
use axum::Json;
use dal::{DalContext, HistoryActor, KeyPair, Tenancy, User, UserPk, Workspace, WorkspacePk};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthConnectRequest {
    pub code: String,
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
struct AuthApiErrBody {
    pub kind: String,
    pub message: String,
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
}

// TODO: pull value from env vars / dotenv files

async fn find_or_create_user_and_workspace(
    mut ctx: DalContext,
    auth_api_user: AuthApiUser,
    auth_api_workspace: AuthApiWorkspace,
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
        Some(workspace) => {
            ctx.update_tenancy(Tenancy::new(*workspace.pk()));
            workspace
        }
        None => {
            let workspace = Workspace::new(
                &mut ctx,
                auth_api_workspace.id,
                auth_api_workspace.display_name,
            )
            .await?;
            let _key_pair = KeyPair::new(&ctx, "default").await?;
            ctx.import_builtins().await?;
            workspace
        }
    };

    // ensure workspace is associated to user
    user.associate_workspace(&ctx, *workspace.pk()).await?;

    ctx.commit().await?;

    Ok((user, workspace))
}

pub async fn auth_connect(
    HandlerContext(builder): HandlerContext,
    Json(request): Json<AuthConnectRequest>,
) -> SessionResult<Json<AuthConnectResponse>> {
    let client = reqwest::Client::new();
    let auth_api_url = match option_env!("LOCAL_AUTH_STACK") {
        Some(_) => "http://localhost:9001",
        None => "https://auth-api.systeminit.com",
    };

    let res = client
        .post(format!("{}/complete-auth-connect", auth_api_url))
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

    let (user, workspace) =
        find_or_create_user_and_workspace(ctx, res_body.user, res_body.workspace).await?;

    Ok(Json(AuthConnectResponse {
        user,
        workspace,
        token: res_body.token,
    }))
}

pub async fn auth_reconnect(
    HandlerContext(builder): HandlerContext,
    RawAccessToken(raw_access_token): RawAccessToken,
) -> SessionResult<Json<AuthReconnectResponse>> {
    let auth_api_url = match option_env!("LOCAL_AUTH_STACK") {
        Some(_) => "http://localhost:9001",
        // None => "https://auth-api.systeminit.com",
        None => "http://localhost:9001",
    };

    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/auth-reconnect", auth_api_url))
        .bearer_auth(&raw_access_token)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        let res_err_body = res
            .json::<AuthApiErrBody>()
            .await
            .map_err(|err| SessionError::AuthApiError(err.to_string()))?;
        println!("reconnect failed = {:?}", res_err_body.message);
        return Err(SessionError::AuthApiError(res_err_body.message));
    }

    let res_body = res.json::<AuthApiReconnectResponse>().await?;

    let ctx = builder.build_default().await?;

    let (user, workspace) =
        find_or_create_user_and_workspace(ctx, res_body.user, res_body.workspace).await?;

    Ok(Json(AuthReconnectResponse { user, workspace }))
}
