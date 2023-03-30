use super::{SessionError, SessionResult};
use crate::server::extract::HandlerContext;
use axum::Json;
use dal::{AccessBuilder, HistoryActor, KeyPair, Tenancy, User, UserPk, Workspace, WorkspacePk};
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

pub async fn auth_connect(
    HandlerContext(builder): HandlerContext,
    Json(request): Json<AuthConnectRequest>,
) -> SessionResult<Json<AuthConnectResponse>> {
    // TODO: pull value from env vars / dotenv files
    let auth_api_url = match option_env!("LOCAL_ENV_STACK") {
        Some(_) => "http://localhost:9001",
        None => "https://auth-api.systeminit.com",
    };

    let client = reqwest::Client::new();
    let res = client
        // TODO: pull this from an env var
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

    let mut ctx = builder
        .build(
            AccessBuilder::new(
                // Empty tenancy means things can be written, but won't ever be read by whatever uses the standard model
                Tenancy::new_empty(),
                HistoryActor::SystemInit,
            )
            .build_head(),
        )
        .await?;
    // lookup user or create if we've never seen it before
    let maybe_user = User::get_by_pk(&ctx, res_body.user.id).await?;
    let user = match maybe_user {
        Some(user) => user,
        None => {
            User::new(
                &ctx,
                res_body.user.id,
                res_body.user.nickname,
                res_body.user.email,
                res_body.user.picture_url,
            )
            .await?
        }
    };
    ctx.update_history_actor(HistoryActor::User(user.pk()));

    // lookup workspace or create if we've never seen it before
    let maybe_workspace = Workspace::get_by_pk(&ctx, &res_body.workspace.id).await?;
    let workspace = match maybe_workspace {
        Some(workspace) => {
            ctx.update_tenancy(Tenancy::new(*workspace.pk()));
            workspace
        }
        None => {
            let workspace = Workspace::new(
                &mut ctx,
                res_body.workspace.id,
                res_body.workspace.display_name,
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

    Ok(Json(AuthConnectResponse {
        user,
        workspace,
        token: res_body.token,
    }))
}
