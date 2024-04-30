use axum::extract::OriginalUri;
use axum::Json;
use dal::func::authoring::{FuncAuthoringClient, TestExecuteFuncResult};
use dal::{ComponentId, FuncId, Visibility};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequest {
    pub id: FuncId,
    pub args: serde_json::Value,
    pub execution_key: String,
    pub code: String,
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ExecuteResponse = TestExecuteFuncResult;

pub async fn execute(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(req): Json<ExecuteRequest>,
) -> FuncResult<Json<ExecuteResponse>> {
    let ctx = builder.build(request_ctx.build(req.visibility)).await?;

    let response = FuncAuthoringClient::test_execute_func(
        &ctx,
        req.id,
        req.args,
        req.execution_key,
        req.code,
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

    Ok(Json(response))
}
