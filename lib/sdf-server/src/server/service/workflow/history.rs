use super::{WorkflowError, WorkflowResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::workflow_runner::workflow_runner_state::{WorkflowRunnerState, WorkflowRunnerStatus};
use dal::{
    StandardModel, Timestamp, Visibility, WorkflowPrototype, WorkflowRunner, WorkflowRunnerId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HistoryWorkflowsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowHistoryView {
    id: WorkflowRunnerId,
    title: String,
    description: Option<String>,
    status: WorkflowRunnerStatus,
    #[serde(flatten)]
    timestamp: Timestamp,
}

pub type HistoryWorkflowsResponse = Vec<WorkflowHistoryView>;

pub async fn history(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<HistoryWorkflowsRequest>,
) -> WorkflowResult<Json<HistoryWorkflowsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let workflows = WorkflowRunner::list(&ctx).await?;

    let mut workflow_views = Vec::new();

    for workflow in workflows {
        let prototype = WorkflowPrototype::get_by_id(&ctx, &workflow.workflow_prototype_id())
            .await?
            .ok_or_else(|| WorkflowError::PrototypeNotFound(workflow.workflow_prototype_id()))?;

        let runner_state = WorkflowRunnerState::find_for_workflow_runner(&ctx, *workflow.id())
            .await?
            .ok_or_else(|| WorkflowError::RunnerStateNotFound(*workflow.id()))?;

        workflow_views.push(WorkflowHistoryView {
            id: *workflow.id(),
            title: prototype.title().to_owned(),
            description: prototype.description().map(ToString::to_string),
            status: runner_state.status(),
            timestamp: *workflow.timestamp(),
        });
    }

    Ok(Json(workflow_views))
}
