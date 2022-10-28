use axum::{extract::Query, Json};
use dal::fix::FixError as DalFixError;
use dal::ComponentError as DalComponentError;
use dal::{
    Component, ComponentId, FixBatch, FixBatchId, FixCompletionStatus, FixId, StandardModel,
    Visibility,
};
use serde::{Deserialize, Serialize};

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::fix::FixError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixHistoryView {
    id: FixId,
    status: FixCompletionStatus,
    action: String,
    component_name: String,
    component_id: ComponentId,
    provider: Option<String>,
    output: Option<String>,
    started_at: String,
    finished_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchHistoryView {
    id: FixBatchId,
    status: FixCompletionStatus,
    author: String,
    fixes: Vec<FixHistoryView>,
    started_at: String,
    finished_at: String,
}

pub type ListResponse = Vec<BatchHistoryView>;

pub async fn list(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListRequest>,
) -> FixResult<Json<ListResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut batch_views = Vec::new();
    for batch in FixBatch::list_finished(&ctx).await? {
        let mut fix_views = Vec::new();
        for fix in batch.fixes(&ctx).await? {
            let component = Component::get_by_id(&ctx, &fix.component_id())
                .await?
                .ok_or_else(|| DalComponentError::NotFound(fix.component_id()))?;

            fix_views.push(FixHistoryView {
                id: *fix.id(),
                status: *fix
                    .completion_status()
                    .ok_or(DalFixError::EmptyCompletionStatus)?,
                action: fix
                    .action()
                    .map(|a| a.to_string())
                    .ok_or_else(|| FixError::MissingAction(*fix.id()))?,
                component_name: component.name(&ctx).await?,
                component_id: *component.id(),
                provider: None,
                output: fix.logs().map(|l| l.to_string()),
                started_at: fix
                    .started_at()
                    .map(|s| s.to_string())
                    .ok_or_else(|| FixError::MissingStartedTimestampForFix(*fix.id()))?,
                finished_at: fix
                    .finished_at()
                    .map(|s| s.to_string())
                    .ok_or_else(|| FixError::MissingFinishedTimestampForFix(*fix.id()))?,
            })
        }
        batch_views.push(BatchHistoryView {
            id: *batch.id(),
            status: *batch
                .completion_status()
                .ok_or(DalFixError::EmptyCompletionStatus)?,
            fixes: fix_views,
            author: batch.author(),
            started_at: batch
                .started_at()
                .map(|s| s.to_string())
                .ok_or_else(|| FixError::MissingStartedTimestampForFixBatch(*batch.id()))?,
            finished_at: batch
                .finished_at()
                .map(|s| s.to_string())
                .ok_or_else(|| FixError::MissingFinishedTimestampForFixBatch(*batch.id()))?,
        })
    }

    Ok(Json(batch_views))
}
