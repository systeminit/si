use super::{WorkflowError, WorkflowResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    Component, Schema, SchemaVariant, StandardModel, Visibility,
    WorkflowPrototypeId, WorkflowRunner,
};
use dal::workflow::HistoryWorkflowStatus;
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
    id: WorkflowPrototypeId,
    title: String,
    description: Option<String>,
    status: HistoryWorkflowStatus,
}

pub type HistoryWorkflowsResponse = Vec<WorkflowHistoryView>;

pub async fn history(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<HistoryWorkflowsRequest>,
) -> WorkflowResult<Json<HistoryWorkflowsResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let workflows = WorkflowRunner::list(&ctx).await?.into_iter().map(|run| WorkflowHistoryView{
        id: *run.id(),
        title: run.title().to_owned(),
        description: run.description().map(ToString::to_string),
        status: HistoryWorkflowStatus::Success, // TODO(wendy) - implement status
    }).collect();

    txns.commit().await?;

    Ok(Json(workflows))
}
