use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::service::change_set::ChangeSetError;
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::change_set::ChangeSet;
use dal::ChangeSetId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbandonChangeSetRequest {
    pub change_set_id: ChangeSetId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbandonChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn abandon_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<AbandonChangeSetRequest>,
) -> ChangeSetResult<()> {
    let mut ctx = builder.build_head(access_builder).await?;

    let mut change_set = ChangeSet::find(&ctx, request.change_set_id)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(change_set.id)
        .await?;
    change_set.abandon(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "abandon_change_set",
        serde_json::json!({
            "abandoned_change_set": request.change_set_id,
        }),
    );

    ctx.commit_no_rebase().await?;

    Ok(())
}
