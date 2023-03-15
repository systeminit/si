use axum::Json;
use serde::{Deserialize, Serialize};

use super::WorkflowResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{job::definition::WorkflowRun, ComponentId, Visibility, WorkflowPrototypeId};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunRequest {
    pub id: WorkflowPrototypeId,
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunResponse {
    run_id: usize,
}

pub async fn run(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<WorkflowRunRequest>,
) -> WorkflowResult<Json<WorkflowRunResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let run_id: usize = rand::random();
    ctx.enqueue_job(WorkflowRun::new(
        &ctx,
        run_id,
        request.id,
        request.component_id,
    ))
    .await;

    ctx.commit().await?;

    Ok(Json(WorkflowRunResponse { run_id }))
}
