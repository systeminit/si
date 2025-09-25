use axum::{
    Json,
    extract::Path,
};
use serde::Serialize;
use serde_json::json;
use si_db::{
    ManagementFuncJobState,
    ManagementState,
};
use si_id::FuncRunId;
use utoipa::ToSchema;

use super::{
    ManagementFuncJobStateV1RequestPath,
    ManagementFuncsError,
    ManagementFuncsResult,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/management-funcs/{management_func_job_state_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("management_func_job_state_id" = String, Path, description = "Management Func Job identifier"),
    ),
    tag = "management_funcs",
    summary = "Get management funcs job state details",
    responses(
        (status = 200, description = "Management Func Job retrieved successfully", body = GetManagementFuncJobStateV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Management Func Job not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_management_func_run_state(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ManagementFuncJobStateV1RequestPath {
        management_func_job_state_id,
    }): Path<ManagementFuncJobStateV1RequestPath>,
) -> ManagementFuncsResult<Json<GetManagementFuncJobStateV1Response>> {
    let management_func_job = ManagementFuncJobState::get_by_id(ctx, management_func_job_state_id)
        .await
        .map_err(|_m| {
            ManagementFuncsError::ManagementFuncJobStateNotFound(management_func_job_state_id)
        })?;

    tracker.track(
        ctx,
        "api_get_management_func_run_state",
        json!({
            "management_func_job_state_id": management_func_job_state_id
        }),
    );

    Ok(Json(GetManagementFuncJobStateV1Response {
        func_run_id: management_func_job.func_run_id(),
        state: management_func_job.state(),
    }))
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetManagementFuncJobStateV1Response {
    #[schema(value_type = Option<String>, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub func_run_id: Option<FuncRunId>,
    #[schema(value_type = String, example = "Executing")]
    pub state: ManagementState,
}
