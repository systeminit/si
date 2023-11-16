use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::change_set::{ChangeSetError, ChangeSetResult};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{ChangeSet, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BeginMergeFlow {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn begin_approval_process(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<BeginMergeFlow>,
) -> ChangeSetResult<Json<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &ctx.visibility().change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.begin_approval_flow(&mut ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "begin_approval_process",
        serde_json::json!({
            "how": "/change_set/begin_approval_process",
            "change_set_pk": ctx.visibility().change_set_pk,
        }),
    );

    WsEvent::change_set_begin_approval_process(&ctx, ctx.visibility().change_set_pk)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
