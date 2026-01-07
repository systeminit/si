use axum::{
    Json,
    response::Json as ResponseJson,
};
use dal::WsEvent;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    PolicyReport,
    PolicyReportResult,
};
use si_id::PolicyReportId;
use utoipa::ToSchema;

use super::PolicyReportsResult;
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

/// The request payload for uploading a policy report
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadPolicyReportV1Request {
    /// The unique name of the policy report.
    pub name: String,
    /// The policy document that was used for evaluation.
    pub policy: String,
    /// The contents of the report.
    pub report: String,
    /// Whether the policy check passed or failed.
    #[schema(value_type = String, example = "Pass")]
    pub result: PolicyReportResult,
}

/// The response payload after uploading a policy report.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadPolicyReportV1Response {
    /// The ID of the created policy report.
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub id: PolicyReportId,
}

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/policy-reports",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier")
    ),
    request_body = UploadPolicyReportV1Request,
    tag = "policy_reports",
    summary = "Upload a policy report",
    responses(
        (status = 200, description = "Policy report uploaded successfully", body = UploadPolicyReportV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub(crate) async fn upload_policy_report(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Json(request): Json<UploadPolicyReportV1Request>,
) -> PolicyReportsResult<ResponseJson<UploadPolicyReportV1Response>> {
    tracker.track(
        ctx,
        "api_upload_policy_report",
        serde_json::json!({
            "name": request.name,
            "result": request.result,
        }),
    );

    let policy_report = match request.result {
        PolicyReportResult::Fail => {
            PolicyReport::new_fail(ctx, request.name, request.policy, request.report).await?
        }
        PolicyReportResult::Pass => {
            PolicyReport::new_pass(ctx, request.name, request.policy, request.report).await?
        }
    };

    WsEvent::policy_uploaded(ctx)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(ResponseJson(UploadPolicyReportV1Response {
        id: policy_report.id,
    }))
}
