use axum::{
    Json,
    extract::Path,
};
use dal::WorkspacePk;
use si_db::FuncRunLogDb;
use si_events::FuncRunId;

use super::get_func_run::FuncRunLogView;
use crate::{
    extract::HandlerContext,
    service::v2::{
        AccessBuilder,
        func::FuncAPIResult,
    },
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRunLogsResponse {
    pub logs: Option<FuncRunLogView>,
}

/// Get logs for a specific function run
///
/// This endpoint returns only the logs for a function run without fetching
/// the entire function run details, which makes it more efficient for
/// monitoring log updates.
pub async fn get_func_run_logs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, func_run_id)): Path<(
        WorkspacePk,
        dal::ChangeSetId,
        FuncRunId,
    )>,
) -> FuncAPIResult<Json<GetFuncRunLogsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // Fetch only the logs for this function run
    let logs = FuncRunLogDb::get_for_func_run_id(&ctx, func_run_id)
        .await?
        .map(|v| v.into());

    Ok(Json(GetFuncRunLogsResponse { logs }))
}
