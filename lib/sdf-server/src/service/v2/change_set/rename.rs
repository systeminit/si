use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use serde::Deserialize;

use super::{Error, Result};
use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    track,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameChangeSetRequest {
    new_name: String,
}

pub async fn rename(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<RenameChangeSetRequest>,
) -> Result<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(Error::ChangeSetNotFound(ctx.change_set_id()))?;

    ChangeSet::rename_change_set(&ctx, change_set_id, &request.new_name).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "rename_change_set",
        serde_json::json!({
            "change_set": change_set_id,
            "new_name": request.new_name,
        }),
    );

    ctx.commit().await?;

    Ok(())
}
