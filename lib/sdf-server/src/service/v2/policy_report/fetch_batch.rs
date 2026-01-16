use axum::{
    Json,
    extract::{
        Path,
        Query,
    },
};
use dal::WorkspacePk;
use sdf_extract::PosthogEventTracker;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::PolicyReport;
use si_id::PolicyReportId;

use super::{
    PolicyReportResult,
    PolicyReportView,
};
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
    /// Name filter for the policy reports.
    name: String,
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
    report: Option<PolicyReportView>,
}

pub(crate) async fn fetch_batch(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
    Path(workspace_id): Path<WorkspacePk>,
    Query(request): Query<Request>,
) -> PolicyReportResult<Json<Response>> {
    let ctx = builder.build_head(access_builder).await?;

    // Get the reports and calculate the total page count.
    let batch = PolicyReport::fetch_batch(&ctx, request.size, request.page, request.name).await?;

    tracker.track(
        &ctx,
        "policy_report_fetch_batch",
        serde_json::json!({
            "workspace_id": workspace_id,
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
    Path((workspace_id, policy_id)): Path<(WorkspacePk, PolicyReportId)>,
) -> PolicyReportResult<Json<SingleResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    // Get the reports and calculate the total page count.
    let report = PolicyReport::fetch_single(&ctx, policy_id).await?;

    tracker.track(
        &ctx,
        "policy_report_fetch_single",
        serde_json::json!({
            "workspace_id": workspace_id,
            "report_id": policy_id,
        }),
    );

    Ok(Json(SingleResponse {
        report: report.map(|r| r.into()),
    }))
}
