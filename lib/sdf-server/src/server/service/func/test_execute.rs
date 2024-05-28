use axum::extract::OriginalUri;
use axum::Json;
use dal::func::authoring::FuncAuthoringClient;
use dal::{ComponentId, FuncId, Visibility};
use serde::{Deserialize, Serialize};
use si_events::FuncRunId;

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestExecuteFuncRequest {
    pub id: FuncId,
    pub args: serde_json::Value,
    pub code: String,
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestExecuteFuncResponse {
    func_run_id: FuncRunId,
}

pub async fn test_execute(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(req): Json<TestExecuteFuncRequest>,
) -> FuncResult<Json<TestExecuteFuncResponse>> {
    let ctx = builder.build(request_ctx.build(req.visibility)).await?;

    let func_run_id = FuncAuthoringClient::test_execute_func(
        &ctx,
        req.id,
        req.args,
        Some(req.code),
        req.component_id,
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "test_execute",
        serde_json::json!({
            "how": "/func/test_execute",
            "id": req.id,
            "component_id": req.component_id,
        }),
    );

    ctx.commit().await?;

    Ok(Json(TestExecuteFuncResponse { func_run_id }))
}
