use axum::Json;
use dal::{RequestContext, Workspace, WorkspaceSignup};
use serde::{Deserialize, Serialize};

use super::{generate_fake_name, TestResult};
use crate::server::extract::{HandlerContext, JwtSecretKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    data: WorkspaceSignup,
}

pub async fn signup(
    HandlerContext(builder): HandlerContext,
    JwtSecretKey(_jwt_secret_key): JwtSecretKey,
) -> TestResult<Json<SignupResponse>> {
    let mut ctx = builder.build(RequestContext::default()).await?;

    let workspace_name = generate_fake_name();
    let user_name = generate_fake_name();
    let user_email = format!("{user_name}@example.com");
    let user_password = "snakes";

    let result = Workspace::signup(
        &mut ctx,
        &workspace_name,
        &user_name,
        &user_email,
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(SignupResponse { data: result }))
}
