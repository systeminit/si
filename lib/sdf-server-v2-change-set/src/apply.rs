use axum::extract::{Host, OriginalUri, Path};
use dal::{ChangeSet, ChangeSetId, WorkspacePk};

use super::Result;
use axum_util::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

pub async fn apply(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<()> {
    let mut ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    ChangeSet::prepare_for_apply(&ctx).await?;

    // We need to run a commit before apply so changes get saved
    ctx.commit().await?;

    ChangeSet::apply_to_base_change_set(&mut ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": change_set_id,
        }),
    );

    // WS Event fires from the dal
    ctx.commit().await?;

    Ok(())
}
