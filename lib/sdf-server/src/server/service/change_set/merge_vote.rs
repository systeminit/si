use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::change_set::{ChangeSetError, ChangeSetResult};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{HistoryActor, User, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MergeVoteRequest {
    pub vote: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn merge_vote(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<MergeVoteRequest>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
            .await?
            .ok_or(ChangeSetError::InvalidUser(*user_pk))?,

        HistoryActor::SystemInit => return Err(ChangeSetError::InvalidUserSystemInit),
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "merge_vote",
        serde_json::json!({
            "how": "/change_set/merge_vote",
            "change_set_pk": ctx.visibility().change_set_pk,
            "user_pk": user.pk(),
            "vote": request.vote,
        }),
    );

    WsEvent::change_set_merge_vote(
        &ctx,
        ctx.visibility().change_set_pk,
        user.pk(),
        request.vote,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
