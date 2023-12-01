use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::service::change_set::ChangeSetError;
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{ChangeSet, ChangeSetPk, HistoryActor, User, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbandonChangeSetRequest {
    pub change_set_pk: ChangeSetPk,
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
) -> ChangeSetResult<Json<AbandonChangeSetResponse>> {
    let mut ctx = builder.build_head(access_builder).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.abandon(&mut ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "abandon_change_set",
        serde_json::json!({
            "abandoned_change_set": request.change_set_pk,
        }),
    );

    ctx.blocking_commit().await?;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
            .await?
            .ok_or(ChangeSetError::InvalidUser(*user_pk))?,

        HistoryActor::SystemInit => return Err(ChangeSetError::InvalidUserSystemInit),
    };

    WsEvent::change_set_abandoned(&ctx, change_set.pk, Some(user.pk()))
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(AbandonChangeSetResponse { change_set }))
}
