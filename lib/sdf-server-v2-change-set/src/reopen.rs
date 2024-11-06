use axum::extract::{Host, OriginalUri, Path};
use dal::{ChangeSet, ChangeSetId, WorkspacePk, WsEvent};

use super::{Error, Result};
use axum_util::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

pub async fn reopen(
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

    //todo(brit): should we guard against re-opening abandoned change sets?
    // this might be helpful if we don't...
    change_set.reopen_change_set(&ctx).await?;

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

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "reject_change_set_apply",
        serde_json::json!({
            "change_set": change_set_id,
        }),
    );

    ctx.commit().await?;

    Ok(())
}
