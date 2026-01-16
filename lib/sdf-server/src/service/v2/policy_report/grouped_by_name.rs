use axum::{
    Json,
    extract::Path,
};
use dal::WorkspacePk;
use sdf_extract::PosthogEventTracker;
use serde::Serialize;
use si_db::PolicyReport;

use super::{
    PolicyReportResult,
    PolicyReportView,
};
use crate::{
    extract::HandlerContext,
    service::v2::AccessBuilder,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    groups: Vec<GroupView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupView {
    name: String,
    results: Vec<PolicyReportView>,
}

pub(crate) async fn grouped_by_name(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    tracker: PosthogEventTracker,
    Path(workspace_id): Path<WorkspacePk>,
) -> PolicyReportResult<Json<Response>> {
    let ctx = builder.build_head(access_builder).await?;

    // Get the grouped reports
    let groups = PolicyReport::fetch_grouped_by_name(&ctx).await?;

    tracker.track(
        &ctx,
        "policy_report_grouped_by_name",
        serde_json::json!({
            "workspace_id": workspace_id,
            "group_count": groups.len(),
        }),
    );

    let group_views: Vec<GroupView> = groups
        .into_iter()
        .map(|group| GroupView {
            name: group.name,
            results: group.results.into_iter().map(|r| r.into()).collect(),
        })
        .collect();

    Ok(Json(Response {
        groups: group_views,
    }))
}
