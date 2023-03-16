use axum::{extract::Query, Json};
use chrono::Utc;
use dal::{
    func::backend::js_command::CommandRunResult, schema::SchemaUiMenu,
    workflow_runner::WorkflowRunnerError, AttributeValueId, Component,
    ComponentError as DalComponentError, ComponentId, FixBatch, FixBatchId, FixCompletionStatus,
    FixId, ResourceView, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use veritech_client::ResourceStatus;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFixesRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixHistoryView {
    id: FixId,
    status: FixCompletionStatus,
    action: String,
    schema_name: String,
    component_name: String,
    component_id: ComponentId,
    attribute_value_id: AttributeValueId,
    provider: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    resource: ResourceView,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchHistoryView {
    pub id: FixBatchId,
    pub status: FixCompletionStatus,
    author: String,
    fixes: Vec<FixHistoryView>,
    started_at: Option<String>,
    finished_at: Option<String>,
}

pub type ListFixesResponse = Vec<BatchHistoryView>;

pub async fn list(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListFixesRequest>,
) -> FixResult<Json<ListFixesResponse>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;
    ctx = ctx.clone_with_delete_visibility();

    let mut batch_views = Vec::new();
    for batch in FixBatch::list(&ctx).await? {
        let mut batch_timedout = false;
        // FIXME(paulo): hardcoding 3 minutes timeout to avoid hiding broken batches forever
        let completion_status = if let Some(status) = batch.completion_status() {
            *status
        } else if Utc::now().signed_duration_since(batch.timestamp().created_at)
            > chrono::Duration::minutes(3)
        {
            batch_timedout = true;
            FixCompletionStatus::Failure
        } else {
            continue;
        };

        let mut fix_views = Vec::new();
        for fix in batch.fixes(&ctx).await? {
            // Technically WorkflowRunner returns a vec of resources, but we only handle one resource at a time
            // It's a technical debt we haven't tackled yet, so let's assume it's only one resource
            let resource = fix
                .workflow_runner(&ctx)
                .await?
                .map(|r| Ok::<_, WorkflowRunnerError>(r.resources()?.pop()))
                .transpose()?
                .flatten();

            let resource = if let Some(resource) = resource {
                resource
            } else if batch_timedout {
                CommandRunResult {
                    status: ResourceStatus::Error,
                    value: None,
                    message: Some("Execution timed-out".to_owned()),
                    // TODO: add propper logs here
                    logs: vec![],
                }
            } else {
                // Note: at least one resource is required for fixes that finished, but it's not clear
                // if we want to break this route if the assumption is incorrect or just hide the fix
                warn!("Fix didn't have any resource: {fix:?}");
                continue;
            };

            let component = Component::get_by_id(&ctx, &fix.component_id())
                .await?
                .ok_or_else(|| DalComponentError::NotFound(fix.component_id()))?;
            let schema = component
                .schema(&ctx)
                .await?
                .ok_or_else(|| DalComponentError::NoSchema(fix.component_id()))?;
            let category = SchemaUiMenu::find_for_schema(&ctx, *schema.id())
                .await?
                .map(|um| um.category().to_string());

            fix_views.push(FixHistoryView {
                id: *fix.id(),
                status: fix
                    .completion_status()
                    .copied()
                    .unwrap_or(FixCompletionStatus::Failure),
                action: fix.action().to_owned(),
                schema_name: schema.name().to_owned(),
                attribute_value_id: *fix.attribute_value_id(),
                component_name: component.name(&ctx).await?,
                component_id: *component.id(),
                provider: category,
                resource: ResourceView::new(resource),
                started_at: fix.started_at().map(|s| s.to_string()),
                finished_at: fix.finished_at().map(|s| s.to_string()),
            })
        }
        batch_views.push(BatchHistoryView {
            id: *batch.id(),
            status: completion_status,
            fixes: fix_views,
            author: batch.author(),
            started_at: batch.started_at().map(|s| s.to_string()),
            finished_at: batch.finished_at().map(|s| s.to_string()),
        })
    }

    Ok(Json(batch_views))
}
