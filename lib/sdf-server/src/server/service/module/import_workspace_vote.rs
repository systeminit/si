use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::pkg::{PkgError, PkgResult};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{HistoryActor, User, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportVoteRequest {
    pub vote: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn import_workspace_vote(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ImportVoteRequest>,
) -> PkgResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
            .await?
            .ok_or(PkgError::InvalidUser(*user_pk))?,

        HistoryActor::SystemInit => {
            return Err(PkgError::InvalidUserSystemInit);
        }
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "merge_vote",
        serde_json::json!({
            "how": "/variant_definition/import_vote",
            "workspace_pk": ctx.tenancy().workspace_pk(),
            "user_pk": user.pk(),
            "vote": request.vote,
        }),
    );

    WsEvent::import_workspace_vote(&ctx, ctx.tenancy().workspace_pk(), user.pk(), request.vote)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
