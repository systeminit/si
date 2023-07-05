use axum::Json;
use serde::{Deserialize, Serialize};

use dal::Workspace;
use telemetry::prelude::*;

use crate::{
    server::extract::{HandlerContext, SignupSecret},
    service::signup::SignupError,
};

use super::SignupResult;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub workspace_name: String,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
    pub signup_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountResponse {
    pub success: bool,
}

pub async fn create_account(
    HandlerContext(builder): HandlerContext,
    SignupSecret(signup_secret): SignupSecret,
    Json(request): Json<CreateAccountRequest>,
) -> SignupResult<Json<CreateAccountResponse>> {
    if signup_secret.as_str() != request.signup_secret.as_str() {
        warn!("invalid signup secret provided when signing up new workspace");
        return Err(SignupError::InvalidSignupSecret);
    }

    let mut ctx = builder.build_default().await?;

    let _nw = Workspace::signup(
        &mut ctx,
        &request.workspace_name,
        &request.user_name,
        &request.user_email,
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(CreateAccountResponse { success: true }))
}
