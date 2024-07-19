use axum::{extract::OriginalUri, Json};
use serde::{Deserialize, Serialize};

use dal::{context::RebaseRequest, ChangeSet, Ulid, Visibility, WsEvent};
use si_events::VectorClockId;

use super::ChangeSetResult;
use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    tracking::track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RebaseOnBaseRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RebaseOnBaseResponse {
    pub rebase_successful: bool,
}

pub async fn rebase_on_base(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RebaseOnBaseRequest>,
) -> ChangeSetResult<Json<RebaseOnBaseResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let change_set = ChangeSet::find(&ctx, request.visibility.change_set_id)
        .await?
        .ok_or(dal::ChangeSetError::ChangeSetNotFound(
            request.visibility.change_set_id,
        ))?;
    let base_change_set = if let Some(base_change_set_id) = change_set.base_change_set_id {
        ChangeSet::find(&ctx, base_change_set_id)
            .await?
            .ok_or(dal::ChangeSetError::ChangeSetNotFound(base_change_set_id))?
    } else {
        return Err(dal::ChangeSetError::NoBaseChangeSet(request.visibility.change_set_id).into());
    };
    let base_snapshot_address = base_change_set
        .workspace_snapshot_address
        .ok_or(dal::ChangeSetError::NoWorkspaceSnapshot(base_change_set.id))?;

    // TODO: Check for affected AttributeValues, and enqueue DVU for them.

    let rebase_request = RebaseRequest {
        to_rebase_change_set_id: request.visibility.change_set_id,
        onto_workspace_snapshot_address: base_snapshot_address,
        // Doesn't really matter, as this field is deprecated since we automatically
        // figure it out in the rebaser.
        onto_vector_clock_id: VectorClockId::new(Ulid::new(), Ulid::new()),
    };

    ctx.do_rebase_request(rebase_request).await?;

    let user = ChangeSet::extract_userid_from_context(&ctx).await;
    // There is no commit, and the rebase request has already gone through & succeeded, so send out
    // the WsEvent immediately.
    WsEvent::change_set_applied(&ctx, base_change_set.id, change_set.id, user)
        .await?
        .publish_immediately(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "rebase_on_base",
        serde_json::json!({
            "rebased_change_set": request.visibility.change_set_id,
        }),
    );

    Ok(Json(RebaseOnBaseResponse {
        rebase_successful: true,
    }))
}
