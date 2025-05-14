use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::ChangeSet;
use sdf_core::tracking::track;
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;

use super::ChangeSetResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BeginMergeFlow {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CancelMergeFlow {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn begin_approval_process(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<BeginMergeFlow>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut change_set = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id).await?;
    change_set.begin_approval_flow(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "begin_approval_process",
        serde_json::json!({
            "how": "/change_set/begin_approval_process",
            "change_set_id": ctx.visibility().change_set_id,
        }),
    );

    ctx.commit_no_rebase().await?;

    Ok(Json(()))
}

pub async fn cancel_approval_process(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CancelMergeFlow>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut change_set = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id).await?;
    change_set.cancel_approval_flow(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "cancel_approval_process",
        serde_json::json!({
            "how": "/change_set/cancel_approval_process",
            "change_set_id": ctx.visibility().change_set_id,
        }),
    );

    ctx.commit_no_rebase().await?;

    Ok(Json(()))
}
