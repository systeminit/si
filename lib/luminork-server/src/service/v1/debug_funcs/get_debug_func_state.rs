use axum::{
    Json,
    extract::Path,
};
use dal::func::debug::{
    DebugFuncJobState,
    DebugFuncJobStateRow,
};
use serde::Serialize;
use serde_json::json;
use si_id::{
    DebugFuncJobStateId,
    FuncRunId,
};
use utoipa::ToSchema;

use super::{
    DebugFuncsError,
    DebugFuncsResult,
    DebugFuncsV1RequestPath,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/debug-funcs/{debug_func_job_state_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("debug_func_job_state_id" = String, Path, description = "Debug Func Job identifier"),
    ),
    tag = "debug_funcs",
    summary = "Get debug funcs job state details",
    responses(
        (status = 200, description = "Debug Function Job retrieved successfully", body = GetDebugFuncJobStateV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Debug Function Job not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_debug_func_state(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(DebugFuncsV1RequestPath {
        debug_func_job_state_id,
    }): Path<DebugFuncsV1RequestPath>,
) -> DebugFuncsResult<Json<GetDebugFuncJobStateV1Response>> {
    let management_func_job = DebugFuncJobStateRow::get_by_id(ctx, debug_func_job_state_id)
        .await
        .map_err(|_m| DebugFuncsError::DebugFuncsJobStateNotFound(debug_func_job_state_id))?;

    tracker.track(
        ctx,
        "api_get_debug_func_state",
        json!({
            "debug_func_job_state_id": debug_func_job_state_id
        }),
    );

    Ok(Json(GetDebugFuncJobStateV1Response {
        id: debug_func_job_state_id,
        func_run_id: management_func_job.func_run_id,
        state: management_func_job.state,
        failure: management_func_job.failure,
        result: management_func_job.result,
    }))
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetDebugFuncJobStateV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub id: DebugFuncJobStateId,
    #[schema(value_type = Option<String>, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub func_run_id: Option<FuncRunId>,
    #[schema(value_type = String, example = "pending")]
    pub state: DebugFuncJobState,
    #[schema(value_type = Option<String>, example = "Could not execute function")]
    pub failure: Option<String>,
    #[schema(value_type = Option<serde_json::Value>, example = "{ \"ami\": \"ami-0abcdef1234567890\" }")]
    pub result: Option<serde_json::Value>,
}
