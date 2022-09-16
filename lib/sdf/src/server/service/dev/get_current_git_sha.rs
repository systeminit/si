use axum::Json;
use dal::{SystemId, Visibility};
use serde::{Deserialize, Serialize};

use super::DevResult;

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

pub async fn get_current_git_sha() -> DevResult<Json<GetCurrentGitShaResponse>> {
    Ok(Json(GetCurrentGitShaResponse {
        sha: CURRENT_GIT_SHA.to_string(),
    }))
}
