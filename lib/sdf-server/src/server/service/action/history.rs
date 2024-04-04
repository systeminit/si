use axum::Json;
use dal::deprecated_action::runner::ActionHistoryView;
use dal::{ActionCompletionStatus, DeprecatedActionBatch, DeprecatedActionBatchId};
use serde::{Deserialize, Serialize};

use super::ActionResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchHistoryView {
    pub id: DeprecatedActionBatchId,
    pub status: Option<ActionCompletionStatus>,
    author: String,
    actors: Option<Vec<String>>,
    actions: Vec<ActionHistoryView>,
    started_at: Option<String>,
    finished_at: Option<String>,
}

pub type ListActionHistoryResponse = Vec<BatchHistoryView>;

pub async fn history(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
) -> ActionResult<Json<ListActionHistoryResponse>> {
    let ctx = builder.build_head(request_ctx).await?;

    let mut batch_views = Vec::new();
    for batch in DeprecatedActionBatch::list(&ctx).await? {
        let completion_status = if let Some(status) = batch.completion_status {
            Some(status)
        } else {
            Some(ActionCompletionStatus::Unstarted)
        };

        let mut action_views = Vec::new();
        let mut runners = batch.runners(&ctx).await?;
        runners.sort_by_key(|f| f.id);

        for runner in runners {
            let history_view = runner.history_view().await?;
            action_views.push(history_view)
        }

        let author = batch.author();
        let action_actors: Option<Vec<String>> = None;

        batch_views.push(BatchHistoryView {
            id: batch.id,
            status: completion_status,
            actions: action_views,
            author,
            actors: action_actors,
            started_at: batch.started_at.as_ref().map(|s| s.to_string()),
            finished_at: batch.finished_at.as_ref().map(|s| s.to_string()),
        })
    }

    Ok(Json(batch_views))
}
