use axum::response::Json;
use dal::{
    Component,
    ComponentError,
    ComponentId,
    Func,
    func::debug::dispatch_debug_func,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_id::DebugFuncJobStateId;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    DebugFuncsError,
    DebugFuncsResult,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/debug-funcs",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    summary = "Execute a debug function in the context of a component",
    tag = "debug_funcs",
    request_body = ExecDebugFuncV1Request,
    responses(
        (status = 200, description = "Debug function execution started", body = ExecDebugFuncV1Response),
        (status = 400, description = "Bad request - Invalid input"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn exec_debug_func(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<ExecDebugFuncV1Request>, axum::extract::rejection::JsonRejection>,
) -> DebugFuncsResult<Json<ExecDebugFuncV1Response>> {
    let Json(payload) = payload?;

    let debug_func = Func::new_debug(&payload.name, &payload.code, &payload.handler);

    let debug_func_job_state_id = match Component::get_by_id(ctx, payload.component_id).await {
        Ok(_) => dispatch_debug_func(ctx, payload.component_id, debug_func, payload.debug_input)
            .await
            .map_err(|e| DebugFuncsError::InternalError(e.to_string()))?,
        Err(ComponentError::NotFound(_)) => {
            return Err(DebugFuncsError::ComponentNotFound(payload.component_id));
        }
        Err(err) => {
            return Err(DebugFuncsError::InternalError(err.to_string()));
        }
    };

    tracker.track(
        ctx,
        "api_exec_debug_func",
        serde_json::json!({
            "func_name": payload.name.to_owned(),
            "component_id": payload.component_id,
            "debug_func_job_state_id": debug_func_job_state_id,
        }),
    );

    ctx.commit_no_rebase()
        .await
        .map_err(|e| DebugFuncsError::InternalError(e.to_string()))?;

    Ok(Json(ExecDebugFuncV1Response {
        debug_func_job_state_id,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecDebugFuncV1Request {
    #[schema(value_type = String, example = "getAmiIdsForRegion")]
    pub name: String,
    #[schema(value_type = String, example = "main")]
    pub handler: String,
    #[schema(value_type = String, example = "async function main() { return 'Hello World'; }")]
    pub code: String,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub component_id: ComponentId,
    #[schema(value_type = Option<serde_json::Value>, example = "{ \"ami\": \"ami-0abcdef1234567890\" }")]
    pub debug_input: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecDebugFuncV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub debug_func_job_state_id: DebugFuncJobStateId,
}
