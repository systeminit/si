use axum::{extract::Query, Json};
use dal::{SystemId, Visibility};
use serde::{Deserialize, Serialize};

use super::DevResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

const CURRENT_GIT_SHA: &str = env!("SI_CURRENT_GIT_SHA");

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCurrentGitShaRequest {
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCurrentGitShaResponse {
    pub sha: String,
}

pub async fn get_current_git_sha(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetCurrentGitShaRequest>,
) -> DevResult<Json<GetCurrentGitShaResponse>> {
    let txns = txns.start().await?;

    let _ctx = builder.build(request_ctx.build(request.visibility), &txns);
    let _system_id = request.system_id.unwrap_or(SystemId::NONE);

    txns.commit().await?;

    Ok(Json(GetCurrentGitShaResponse {
        sha: CURRENT_GIT_SHA.to_string(),
    }))
}
