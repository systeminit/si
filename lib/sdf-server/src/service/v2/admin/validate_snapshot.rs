use axum::{
    Json,
    extract::Path,
};
use dal::{
    AttributeValue,
    ChangeSetId,
    DalContext,
    WorkspacePk,
    attribute::prototype::argument::AttributePrototypeArgument,
    workspace_snapshot::graph::validator::ValidationIssue,
};
use si_db::Tenancy;

use crate::service::v2::admin::{
    AdminAPIResult,
    AdminUserContext,
};

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
    pub fixed: bool,
}

pub async fn validate_snapshot(
    AdminUserContext(mut ctx): AdminUserContext,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<Json<Response>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));
    ctx.update_visibility_and_snapshot_to_visibility(change_set_id)
        .await?;
    let issues = get_validation_issues(&ctx).await?;

    Ok(Json(Response { issues }))
}

pub async fn validate_and_fix_snapshot(
    AdminUserContext(mut ctx): AdminUserContext,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<Json<Response>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));
    ctx.update_visibility_and_snapshot_to_visibility(change_set_id)
        .await?;
    let mut issues = get_validation_issues(&ctx).await?;
    for issue in &mut issues {
        issue.fixed = fix_issue(&ctx, &issue.issue).await?;
    }
    ctx.commit().await?;

    Ok(Json(Response { issues }))
}

async fn get_validation_issues(
    ctx: &DalContext,
) -> AdminAPIResult<Vec<ValidationIssueWithMessage>> {
    Ok(ctx
        .workspace_snapshot()?
        .as_legacy_snapshot()?
        .validate()
        .await?
        .into_iter()
        .map(|(issue, message)| ValidationIssueWithMessage {
            issue,
            message,
            fixed: false,
        })
        .collect())
}

async fn fix_issue(ctx: &DalContext, issue: &ValidationIssue) -> AdminAPIResult<bool> {
    Ok(match issue {
        &ValidationIssue::ConnectionToMissingComponent { apa, .. }
        | &ValidationIssue::ConnectionToUnknownSocket { apa, .. } => {
            // These will never be fixed, so we just remove them
            AttributePrototypeArgument::remove(ctx, apa).await?;
            true
        }
        &ValidationIssue::MissingValue { .. } => {
            // We can only remove this if it is a connection from an input socket, meaning it has
            // targets and is hanging off an input socket.
            false
        }
        &ValidationIssue::DuplicateAttributeValue { duplicate, .. }
        | &ValidationIssue::DuplicateAttributeValueWithDifferentValues { duplicate, .. } => {
            // These are extra, so we remove them (which will also enqueue subscribers to DVU!)
            AttributeValue::remove(ctx, duplicate).await?;
            true
        }
        ValidationIssue::ArgumentTargets { .. }
        | ValidationIssue::ComponentHasParent { .. }
        | ValidationIssue::MissingChildAttributeValues { .. }
        | ValidationIssue::MultipleValues { .. }
        | ValidationIssue::UnknownChildAttributeValue { .. } => false,
    })
}
