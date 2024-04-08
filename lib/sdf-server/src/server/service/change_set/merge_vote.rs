use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::service::change_set::ChangeSetError;
use crate::service::change_set::ChangeSetResult;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{ChangeSet, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MergeVoteRequest {
    pub vote: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn merge_vote(
    OriginalUri(_original_uri): OriginalUri,
    PosthogClient(_posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<MergeVoteRequest>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut change_set = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.merge_vote(&ctx, request.vote).await?;

    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "merge_vote",
    //     serde_json::json!({
    //         "how": "/change_set/merge_vote",
    //         "change_set_pk": ctx.visibility().change_set_pk,
    //         "user_pk": user.pk(),
    //         "vote": request.vote,
    //     }),
    // );

    ctx.commit_no_rebase().await?;

    Ok(Json(()))
}
