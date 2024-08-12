use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::change_set::{ChangeSetError, ChangeSetResult};
use axum::extract::{Host, OriginalUri};
use axum::Json;
use dal::{ChangeSet, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BeginAbandonFlow {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CancelAbandonFlow {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn begin_abandon_approval_process(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<BeginAbandonFlow>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let maybe_head_changeset = ctx.get_workspace_default_change_set_id().await?;
    if maybe_head_changeset == request.visibility.change_set_id {
        return Err(ChangeSetError::CannotAbandonHead);
    }
    let mut change_set = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;

    change_set.begin_abandon_approval_flow(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "begin_abandon_approval_process",
        serde_json::json!({
            "how": "/change_set/begin_abandon_approval_process",
            "change_set_id": ctx.visibility().change_set_id,
        }),
    );
    ctx.commit_no_rebase().await?;
    Ok(Json(()))
}

pub async fn cancel_abandon_approval_process(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CancelAbandonFlow>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut change_set = ChangeSet::find(&ctx, ctx.change_set_id())
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.cancel_abandon_approval_flow(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "cancel_abandon_approval_process",
        serde_json::json!({
            "how": "/change_set/cancel_abandon_approval_process",
            "change_set_id": ctx.visibility().change_set_id,
        }),
    );

    ctx.commit_no_rebase().await?;

    Ok(Json(()))
}
