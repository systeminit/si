use axum::{
    Json,
    extract::{
        Path,
        Query,
    },
};
use dal::{
    ChangeSetId,
    UserPk,
    WorkspacePk,
};
use sdf_extract::PosthogEventTracker;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::PolicyReport;
use si_id::PolicyReportId;

use super::PolicyReportResult;
use crate::{
    extract::HandlerContext,
    service::v2::AccessBuilder,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Request {
    /// The page size, which is the number of reports per page.
    size: Option<u64>,
    /// The page number, which is "1-indexed".
    page: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    reports: Vec<PolicyReportView>,
    page_size: u64,
    page_number: u64,
    total_page_count: u64,
    total_report_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SingleResponse {
    report: Option<PolicyReport>,
}

// NOTE(nick): we need this to convert to camelcase and to handle timestamp conversion.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyReportView {
    id: PolicyReportId,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    user_id: Option<UserPk>,
    created_at: String,
    name: String,
    policy: String,
    report: String,
    result: si_db::PolicyReportResult,
}

impl From<PolicyReport> for PolicyReportView {
    fn from(report: PolicyReport) -> Self {
        Self {
            id: report.id,
            workspace_id: report.workspace_id,
            change_set_id: report.change_set_id,
            user_id: report.user_id,
            created_at: report.created_at.to_rfc3339(),
            name: report.name,
            policy: report.policy,
            report: report.report,
            result: report.result,
        }
    }
}

pub(crate) async fn fetch_batch(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<Request>,
) -> PolicyReportResult<Json<Response>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // Get the reports and calculate the total page count.
    let batch = PolicyReport::fetch_batch(&ctx, request.size, request.page).await?;

    tracker.track(
        &ctx,
        "policy_report_fetch_batch",
        serde_json::json!({
            "change_set_id": ctx.change_set_id(),
            "report_count": batch.reports.len(),
            "page_size": batch.page_size,
            "page_number": batch.page_number,
            "total_page_count": batch.total_page_count,
            "total_report_count": batch.total_report_count,
        }),
    );

    Ok(Json(Response {
        reports: batch.reports.into_iter().map(|r| r.into()).collect(),
        page_size: batch.page_size,
        page_number: batch.page_number,
        total_page_count: batch.total_page_count,
        total_report_count: batch.total_report_count,
    }))
}

pub(crate) async fn fetch_single(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, policy_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        PolicyReportId,
    )>,
) -> PolicyReportResult<Json<SingleResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // Get the reports and calculate the total page count.
    let report = PolicyReport::fetch_single(&ctx, policy_id).await?;

    tracker.track(
        &ctx,
        "policy_report_fetch_single",
        serde_json::json!({
            "change_set_id": ctx.change_set_id(),
            "report_id": policy_id,
        }),
    );

    Ok(Json(SingleResponse { report }))
}
