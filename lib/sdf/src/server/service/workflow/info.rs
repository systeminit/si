use super::{WorkflowError, WorkflowResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::workflow_runner::workflow_runner_state::{WorkflowRunnerState, WorkflowRunnerStatus};
use dal::{
    FuncBindingReturnValue, StandardModel, Timestamp, Visibility, WorkflowPrototype,
    WorkflowRunner, WorkflowRunnerId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunInfoRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub id: WorkflowRunnerId,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunInfoView {
    id: WorkflowRunnerId,
    title: String,
    description: Option<String>,
    status: WorkflowRunnerStatus,
    #[serde(flatten)]
    timestamp: Timestamp,
    logs: Vec<String>,
}

pub type WorkflowRunInfoResponse = WorkflowRunInfoView;

pub async fn info(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<WorkflowRunInfoRequest>,
) -> WorkflowResult<Json<WorkflowRunInfoResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let runner = WorkflowRunner::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(WorkflowError::RunnerNotFound(request.id))?;
    let prototype = WorkflowPrototype::get_by_id(&ctx, &runner.workflow_prototype_id())
        .await?
        .ok_or_else(|| WorkflowError::PrototypeNotFound(runner.workflow_prototype_id()))?;

    let func_binding_return_value =
        FuncBindingReturnValue::get_by_func_binding_id(&ctx, runner.func_binding_id())
            .await?
            .ok_or_else(|| WorkflowError::FuncBindingNotFound(runner.func_binding_id()))?;

    let mut logs = Vec::new();

    for stream in func_binding_return_value
        .get_output_stream(&ctx)
        .await?
        .unwrap_or_default()
    {
        match stream.data {
            Some(data) => logs.push(format!(
                "{} {}",
                stream.message,
                serde_json::to_string_pretty(&data)?
            )),
            None => logs.push(stream.message),
        }
    }

    let runner_state = WorkflowRunnerState::find_for_workflow_runner(&ctx, *runner.id())
        .await?
        .ok_or_else(|| WorkflowError::RunnerStateNotFound(*runner.id()))?;

    let view = WorkflowRunInfoView {
        id: *runner.id(),
        title: prototype.title().to_owned(),
        description: prototype.description().map(ToString::to_string),
        status: runner_state.status(),
        timestamp: *runner.timestamp(),
        logs,
    };

    Ok(Json(view))
}
