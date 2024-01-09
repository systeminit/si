use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::change_set::{ChangeSetError, ChangeSetResult};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{ChangeSet, HistoryActor, User, Visibility, WsEvent};
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
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<BeginAbandonFlow>,
) -> ChangeSetResult<Json<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &ctx.visibility().change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.begin_abandon_approval_flow(&mut ctx).await?;

    let user_pk = match ctx.history_actor() {
        HistoryActor::User(user_pk) => {
            let user = User::get_by_pk(&ctx, *user_pk)
                .await?
                .ok_or(ChangeSetError::InvalidUser(*user_pk))?;

            Some(user.pk())
        }

        HistoryActor::SystemInit => None,
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "begin_abandon_approval_process",
        serde_json::json!({
            "how": "/change_set/begin_abandon_approval_process",
            "change_set_pk": ctx.visibility().change_set_pk,
        }),
    );

    WsEvent::change_set_begin_abandon_approval_process(
        &ctx,
        ctx.visibility().change_set_pk,
        user_pk,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    WsEvent::change_set_abandon_vote(
        &ctx,
        ctx.visibility().change_set_pk,
        user_pk.expect("A user was definitely found as per above"),
        "Approve".to_string(),
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(Json(()))
}

pub async fn cancel_abandon_approval_process(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CancelAbandonFlow>,
) -> ChangeSetResult<Json<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &ctx.visibility().change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.cancel_abandon_approval_flow(&mut ctx).await?;

    let user_pk = match ctx.history_actor() {
        HistoryActor::User(user_pk) => {
            let user = User::get_by_pk(&ctx, *user_pk)
                .await?
                .ok_or(ChangeSetError::InvalidUser(*user_pk))?;

            Some(user.pk())
        }

        HistoryActor::SystemInit => None,
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "cancel_abandon_approval_process",
        serde_json::json!({
            "how": "/change_set/cancel_abandon_approval_process",
            "change_set_pk": ctx.visibility().change_set_pk,
        }),
    );

    WsEvent::change_set_cancel_abandon_approval_process(
        &ctx,
        ctx.visibility().change_set_pk,
        user_pk,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
