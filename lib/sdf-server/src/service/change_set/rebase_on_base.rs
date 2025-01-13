use std::sync::Arc;

use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{ChangeSet, Visibility, WorkspaceSnapshot, WsEvent};
use serde::{Deserialize, Serialize};

use super::ChangeSetResult;
use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    track,
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
    Host(host_name): Host,
    Json(request): Json<RebaseOnBaseRequest>,
) -> ChangeSetResult<Json<RebaseOnBaseResponse>> {
    let ctx: dal::DalContext = builder.build(request_ctx.build(request.visibility)).await?;

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
        return Err(dal::ChangeSetError::NoBaseChangeSet(ctx.change_set_id()).into());
    };

    let base_snapshot = WorkspaceSnapshot::find_for_change_set(&ctx, base_change_set.id).await?;
    if let Some(rebase_batch) = WorkspaceSnapshot::calculate_rebase_batch(
        ctx.workspace_snapshot()?,
        Arc::new(base_snapshot),
    )
    .await?
    {
        let updates_address = ctx.write_rebase_batch(rebase_batch).await?;

        ctx.run_rebase_with_reply(ctx.workspace_pk()?, ctx.change_set_id(), updates_address)
            .await?;
    }

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
        &host_name,
        "rebase_on_base",
        serde_json::json!({
            "rebased_change_set": request.visibility.change_set_id,
        }),
    );

    Ok(Json(RebaseOnBaseResponse {
        rebase_successful: true,
    }))
}
