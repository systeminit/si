use super::{WorkflowError, WorkflowResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    Component, Schema, SchemaVariant, StandardModel, Visibility, WorkflowPrototype,
    WorkflowPrototypeId, WorkflowRunnerId, WorkflowRunner, FuncBindingReturnValue,
};
use dal::workflow::HistoryWorkflowStatus;
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
    id: WorkflowPrototypeId,
    title: String,
    description: Option<String>,
    status: HistoryWorkflowStatus,
    #[serde(flatten)]
    timestamp: Timestamp,
    logs: Vec<String>,
}

pub type WorkflowRunInfoResponse = WorkflowRunInfoView;

pub async fn info(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<WorkflowRunInfoRequest>,
) -> WorkflowResult<Json<WorkflowRunInfoResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let runner = WorkflowRunner::get_by_id(&ctx, &request.id).await?.ok_or(WorkflowError::RunnerNotFound(request.id))?;

    let func_binding_return_value = FuncBindingReturnValue::get_by_func_binding_id(&ctx, runner.func_binding_id()).await?.ok_or(WorkflowError::FuncBindingNotFound(runner.func_binding_id()))?;

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

    let view = WorkflowRunInfoView{
        id: *runner.id(),
        title: runner.title().to_owned(),
        description: runner.description().map(ToString::to_string),
        status: HistoryWorkflowStatus::Success,
        timestamp: *runner.timestamp(),
        logs,
    };

    txns.commit().await?;

    Ok(Json(view))
}
