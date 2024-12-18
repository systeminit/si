use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{
    func::authoring::FuncAuthoringClient, ChangeSetId, ComponentId, Func, FuncId, WorkspacePk,
};
use serde::{Deserialize, Serialize};
use si_events::{audit_log::AuditLogKind, FuncRunId};

use super::FuncAPIResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestExecuteFuncRequest {
    pub args: serde_json::Value,
    pub code: String,
    pub component_id: ComponentId,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestExecuteFuncResponse {
    func_run_id: FuncRunId,
}

pub async fn test_execute(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<TestExecuteFuncRequest>,
) -> FuncAPIResult<Json<TestExecuteFuncResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let func = Func::get_by_id_or_error(&ctx, func_id).await?;
    let func_run_id = FuncAuthoringClient::test_execute_func(
        &ctx,
        func_id,
        request.args,
        Some(request.code),
        request.component_id,
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "test_execute",
        serde_json::json!({
            "how": "/func/test_execute",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "component_id": request.component_id,
        }),
    );
    ctx.write_audit_log(
        AuditLogKind::TestFunction {
            func_id,
            func_display_name: func.display_name,
            func_run_id,
        },
        func.name.clone(),
    )
    .await?;
    ctx.commit().await?;

    Ok(Json(TestExecuteFuncResponse { func_run_id }))
}
