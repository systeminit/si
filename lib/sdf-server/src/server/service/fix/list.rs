use axum::{extract::Query, Json};
use chrono::Utc;
use dal::fix::FixHistoryView;
use dal::{FixBatch, FixBatchId, FixCompletionStatus};
use dal::{StandardModel, Visibility};
use serde::{Deserialize, Serialize};

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFixesRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchHistoryView {
    pub id: FixBatchId,
    pub status: Option<FixCompletionStatus>,
    author: String,
    actors: Option<Vec<String>>,
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
        let mut batch_timed_out = false;
        // FIXME(paulo): hardcoding 5 minutes timeout to avoid hiding broken batches forever
        let completion_status = if let Some(status) = batch.completion_status() {
            Some(*status)
        } else if Utc::now().signed_duration_since(batch.timestamp().created_at)
            > chrono::Duration::minutes(5)
        {
            batch_timed_out = true;
            Some(FixCompletionStatus::Failure)
        } else {
            Some(FixCompletionStatus::Unstarted)
        };

        let mut fix_views = Vec::new();
        let mut fixes = batch.fixes(&ctx).await?;
        fixes.sort_by_key(|f| *f.id());

        for fix in fixes {
            if let Some(history_view) = fix.history_view(&ctx, batch_timed_out).await? {
                fix_views.push(history_view)
            }
        }

        let author = batch.author();
        let mut fix_actors: Option<Vec<String>> = None;
        {
            if let Some(actors) = batch.actors() {
                let mut full_actors: Vec<String> =
                    actors.split(',').map(|s| s.trim().to_string()).collect();
                full_actors.retain(|a| *a != author);
                fix_actors = Some(full_actors);
            }
        }

        batch_views.push(BatchHistoryView {
            id: *batch.id(),
            status: completion_status,
            fixes: fix_views,
            author,
            actors: fix_actors,
            started_at: batch.started_at().map(|s| s.to_string()),
            finished_at: batch.finished_at().map(|s| s.to_string()),
        })
    }

    Ok(Json(batch_views))
}
