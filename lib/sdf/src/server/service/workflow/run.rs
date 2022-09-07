use axum::Json;
use serde::{Deserialize, Serialize};

use super::WorkflowResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{
    workflow_runner::workflow_runner_state::WorkflowRunnerState, ComponentId, Visibility,
    WorkflowPrototypeId, WorkflowRunner,
};

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
    logs: Vec<String>,
    workflow_runner_state: WorkflowRunnerState,
}

pub async fn run(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<WorkflowRunRequest>,
) -> WorkflowResult<Json<WorkflowRunResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // NOTE(nick,wendy): this looks similar to code insider WorkflowRunner::run(). Do we need to run
    // it twice?
    // reference: https://github.com/systeminit/si/blob/87c5cce99d6b972f441358295bbabe27f1d787da/lib/dal/src/workflow_runner.rs#L209-L227
    let (_, workflow_runner_state, func_binding_return_values) =
        WorkflowRunner::run(&ctx, request.id, request.component_id).await?;
    let mut logs = Vec::new();
    for func_binding_return_value in func_binding_return_values {
        for stream in func_binding_return_value
            .get_output_stream(&ctx)
            .await?
            .unwrap_or_default()
        {
            match stream.data {
                Some(data) => logs.push((
                    stream.timestamp,
                    format!(
                        "{} {}",
                        stream.message,
                        serde_json::to_string_pretty(&data)?
                    ),
                )),
                None => logs.push((stream.timestamp, stream.message)),
            }
        }
    }
    logs.sort_by_key(|(timestamp, _)| *timestamp);
    let logs = logs.into_iter().map(|(_, log)| log).collect();

    ctx.commit().await?;

    Ok(Json(WorkflowRunResponse {
        logs,
        workflow_runner_state,
    }))
}
