use axum::{
    Json,
    extract::{
        Path,
        Query,
    },
};
use dal::DalContext;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::FuncRunDb;
use si_events::{
    CasValue,
    FuncRun,
    FuncRunId,
    WorkspacePk,
};

use super::get_func_run::FuncRunView;
use crate::{
    extract::HandlerContext,
    service::v2::{
        AccessBuilder,
        func::FuncAPIResult,
    },
};

/// Get a FuncRunView without fetching logs
///
/// This is an optimized version of get_func_run_view that skips fetching logs
/// to improve performance for paginated views where logs are not immediately needed.
async fn get_func_run_view_without_logs(
    ctx: &DalContext,
    func_run: &FuncRun,
) -> FuncAPIResult<FuncRunView> {
    let arguments: Option<CasValue> = ctx
        .layer_db()
        .cas()
        .try_read_as(&func_run.function_args_cas_address())
        .await?;
    let func_args: serde_json::Value = match arguments {
        Some(func_args_cas_value) => func_args_cas_value.into(),
        None => serde_json::Value::Null,
    };

    let code: Option<CasValue> = ctx
        .layer_db()
        .cas()
        .try_read_as(&func_run.function_code_cas_address())
        .await?;
    let code_base64: String = match code {
        Some(code_base64_cas_value) => {
            let code_base64_cas_value: serde_json::Value = code_base64_cas_value.into();
            match code_base64_cas_value.as_str() {
                Some(code_base64_str) => code_base64_str.to_string(),
                None => "".to_string(),
            }
        }
        None => "".to_string(),
    };

    let result_value: Option<serde_json::Value> = {
        match func_run.result_value_cas_address() {
            Some(result_value_cas_address) => {
                let result_value_cas: Option<CasValue> = ctx
                    .layer_db()
                    .cas()
                    .try_read_as(&result_value_cas_address)
                    .await?;
                result_value_cas.map(|r| r.into())
            }
            None => None,
        }
    };
    let unprocessed_result_value: Option<serde_json::Value> = {
        match func_run.result_unprocessed_value_cas_address() {
            Some(result_unprocessed_value_cas_address) => {
                let result_unprocessed_value_cas: Option<CasValue> = ctx
                    .layer_db()
                    .cas()
                    .try_read_as(&result_unprocessed_value_cas_address)
                    .await?;
                result_unprocessed_value_cas.map(|r| r.into())
            }
            None => None,
        }
    };

    // Skip fetching logs to improve performance
    let logs = None;

    Ok(FuncRunView::new(
        func_run,
        func_args,
        code_base64,
        result_value,
        logs,
        unprocessed_result_value,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    limit: Option<u32>,
    cursor: Option<FuncRunId>,
    component_id: Option<dal::ComponentId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRunsPaginatedResponse {
    func_runs: Vec<FuncRunView>,
    next_cursor: Option<FuncRunId>,
}

/// Get paginated function runs for a workspace
///
/// This endpoint supports cursor-based pagination:
/// - `limit` parameter controls how many items to return per page (default: 50, max: 100)
/// - `cursor` parameter should be the ID of the last item from the previous page
/// - `component_id` parameter filters results to a specific component (optional)
///
/// Results are ordered by creation time (newest first).
pub async fn get_func_runs_paginated(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, dal::ChangeSetId)>,
    Query(params): Query<PaginationParams>,
) -> FuncAPIResult<Json<GetFuncRunsPaginatedResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // Set default limit and enforce a max limit
    let limit = params.limit.unwrap_or(50).min(100);

    // Query the database with pagination parameters
    let func_runs = if let Some(component_id) = params.component_id {
        // Component-specific query
        FuncRunDb::read_many_for_component_paginated(
            &ctx,
            workspace_pk,
            change_set_id,
            component_id,
            limit as i64,
            params.cursor,
        )
        .await?
    } else {
        FuncRunDb::read_many_for_workspace_paginated(
            &ctx,
            workspace_pk,
            change_set_id,
            limit as i64,
            params.cursor,
        )
        .await?
    };

    // Determine the next cursor (if we have at least `limit` results)
    let next_cursor = if func_runs.len() == limit as usize {
        func_runs.last().map(|run| run.id())
    } else {
        None
    };

    // Convert the func runs to views without logs to improve performance
    let mut func_run_views = Vec::with_capacity(func_runs.len());
    for func_run in func_runs {
        let func_run_view = get_func_run_view_without_logs(&ctx, &func_run).await?;
        func_run_views.push(func_run_view);
    }

    Ok(Json(GetFuncRunsPaginatedResponse {
        func_runs: func_run_views,
        next_cursor,
    }))
}
