use axum::{
    Json,
    extract::Path,
};
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use super::{
    FuncRunV1RequestPath,
    FuncsError,
    FuncsResult,
};
use crate::{
    api_types::func_run::v1::FuncRunViewV1,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/runs/{func_run_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("func_run_id", description = "Func run identifier"),
    ),
    tag = "funcs",
    responses(
        (status = 200, description = "Func Run retrieved successfully", body = GetFuncRunV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Func run not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_func_run(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(FuncRunV1RequestPath { func_run_id }): Path<FuncRunV1RequestPath>,
) -> FuncsResult<Json<GetFuncRunV1Response>> {
    let maybe_func_run = ctx.layer_db().func_run().read(func_run_id).await?;
    match maybe_func_run {
        Some(func_run) => {
            let func_run_view = FuncRunViewV1::assemble(ctx, &func_run).await?;

            tracker.track(
                ctx,
                "api_get_func_run",
                json!({
                    "func_run_id": func_run_id
                }),
            );

            Ok(Json(GetFuncRunV1Response {
                func_run: func_run_view,
            }))
        }
        None => Err(FuncsError::FuncRunNotFound(func_run_id)),
    }
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRunV1Response {
    pub func_run: FuncRunViewV1,
}
