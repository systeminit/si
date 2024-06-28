use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use serde::{Deserialize, Serialize};

use dal::{
    func::authoring::FuncAuthoringClient, ChangeSetId, ComponentId, Func, FuncId, WorkspacePk,
};
use si_events::FuncRunId;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    tracking::track,
};

use super::FuncAPIResult;

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
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<TestExecuteFuncRequest>,
) -> FuncAPIResult<Json<TestExecuteFuncResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    // should we force a changeset to test execute?
    // let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

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
        "test_execute",
        serde_json::json!({
            "how": "/func/test_execute",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "component_id": request.component_id,
        }),
    );

    ctx.commit().await?;

    Ok(Json(TestExecuteFuncResponse { func_run_id }))
}
