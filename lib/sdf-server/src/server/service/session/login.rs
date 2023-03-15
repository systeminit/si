use super::SessionError;
use super::SessionResult;
use crate::server::extract::{HandlerContext, JwtSecretKey};
use axum::Json;
use dal::{context::AccessBuilder, HistoryActor, Tenancy, User, Workspace};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub workspace_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user: User,
    pub workspace: Workspace,
    pub jwt: String,
}

pub async fn login(
    HandlerContext(builder): HandlerContext,
    JwtSecretKey(jwt_secret_key): JwtSecretKey,
    Json(request): Json<LoginRequest>,
) -> SessionResult<Json<LoginResponse>> {
    // Global history actor
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

    let workspace = Workspace::find_by_name(&ctx, &request.workspace_name)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    ctx.update_tenancy(Tenancy::new(*workspace.pk()));

    let user = User::find_by_email(&ctx, &request.user_email)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    // Update context history actor
    ctx.update_history_actor(HistoryActor::User(user.pk()));

    let jwt = user
        .login(&ctx, &jwt_secret_key, &request.user_password)
        .await
        .map_err(|_| SessionError::LoginFailed)?;

    Ok(Json(LoginResponse {
        jwt,
        user,
        workspace,
    }))
}
