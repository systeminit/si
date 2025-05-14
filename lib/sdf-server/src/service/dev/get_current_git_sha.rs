use axum::Json;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;

use super::DevResult;

const CURRENT_GIT_SHA: &str = "unset-git-sha";

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCurrentGitShaRequest {
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
