use axum::Json;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::ChangeSetResult;
use crate::extract::{v1::AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusWithBaseRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusWithBaseResponse {
    pub base_has_updates: bool,
    pub change_set_has_updates: bool,
    pub conflicts_with_base: bool,
}

pub async fn status_with_base(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<StatusWithBaseRequest>,
) -> ChangeSetResult<Json<StatusWithBaseResponse>> {
    let _ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // let change_set = ChangeSet::find(&ctx, request.visibility.change_set_id)
    //     .await?
    //     .ok_or(dal::ChangeSetError::ChangeSetNotFound(
    //         request.visibility.change_set_id,
    //     ))?;
    // let cs_workspace_snapshot = WorkspaceSnapshot::find_for_change_set(&ctx, change_set.id).await?;
    // let cs_vector_clock_id = cs_workspace_snapshot
    //     .max_recently_seen_clock_id(Some(change_set.id))
    //     .await?
    //     .ok_or(WorkspaceSnapshotError::RecentlySeenClocksMissing(
    //         change_set.id,
    //     ))?;
    // let base_change_set = if let Some(base_change_set_id) = change_set.base_change_set_id {
    //     ChangeSet::find(&ctx, base_change_set_id)
    //         .await?
    //         .ok_or(dal::ChangeSetError::ChangeSetNotFound(base_change_set_id))?
    // } else {
    //     return Err(dal::ChangeSetError::NoBaseChangeSet(request.visibility.change_set_id).into());
    // };
    // let base_snapshot = WorkspaceSnapshot::find_for_change_set(&ctx, base_change_set.id).await?;
    // let base_vector_clock_id = base_snapshot
    //     .max_recently_seen_clock_id(Some(base_change_set.id))
    //     .await?
    //     .ok_or(WorkspaceSnapshotError::RecentlySeenClocksMissing(
    //         base_change_set.id,
    //     ))?;
    // let conflicts_and_updates_change_set_into_base = base_snapshot
    //     .detect_conflicts_and_updates(
    //         base_vector_clock_id,
    //         &cs_workspace_snapshot,
    //         cs_vector_clock_id,
    //     )
    //     .await?;
    // let conflicts_and_updates_base_into_change_set = cs_workspace_snapshot
    //     .detect_conflicts_and_updates(cs_vector_clock_id, &base_snapshot, base_vector_clock_id)
    //     .await?;

    Ok(Json(StatusWithBaseResponse {
        base_has_updates: false,
        change_set_has_updates: false,
        conflicts_with_base: false,
    }))
}
