use axum::Json;
use dal::workspace_snapshot::graph::validator::ValidationIssue;
use sdf_extract::change_set::ChangeSetDalContext;

use super::Result;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub issues: Vec<ValidationIssueWithMessage>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssueWithMessage {
    #[serde(flatten)]
    pub issue: ValidationIssue,
    pub message: String,
}

pub async fn validate_snapshot(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
) -> Result<Json<Response>> {
    let snapshot = ctx.workspace_snapshot()?.as_legacy_snapshot()?;
    let issues = snapshot
        .validate()
        .await?
        .into_iter()
        .map(|(issue, message)| ValidationIssueWithMessage { issue, message })
        .collect();
    Ok(Json(Response { issues }))
}
