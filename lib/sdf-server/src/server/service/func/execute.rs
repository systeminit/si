use axum::Json;
use dal::func::authoring::{DummyExecutionResult, FuncAuthoringClient};
use dal::{ComponentId, FuncId, Visibility};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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

pub type ExecuteResponse = DummyExecutionResult;

pub async fn execute(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(req): Json<ExecuteRequest>,
) -> FuncResult<Json<ExecuteResponse>> {
    let ctx = builder.build(request_ctx.build(req.visibility)).await?;

    let response = FuncAuthoringClient::dummy_execute_func(
        &ctx,
        req.id,
        req.args,
        req.execution_key,
        req.code,
        req.component_id,
    )
    .await?;

    Ok(Json(response))
}
