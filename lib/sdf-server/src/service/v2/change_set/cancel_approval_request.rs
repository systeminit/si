use axum::extract::{Host, OriginalUri, Path};
use dal::{ChangeSet, ChangeSetId, WorkspacePk, WsEvent};

use super::{Error, Result};
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

pub async fn cancel_approval_request(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    let mut change_set = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(Error::ChangeSetNotFound(ctx.change_set_id()))?;
    let old_status = change_set.status;

    change_set.reopen_change_set(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "approve_change_set_apply",
        serde_json::json!({
            "merged_change_set": change_set_id,
        }),
    );

    WsEvent::change_set_status_changed(
        &ctx,
        change_set.id,
        ChangeSet::extract_userid_from_context(&ctx).await,
        old_status,
        change_set.status,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(())
}
