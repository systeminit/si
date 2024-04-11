use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{ActionId, DeprecatedAction, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveActionRequest {
    pub id: ActionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn remove_action(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RemoveActionRequest>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let action = DeprecatedAction::get_by_id(&ctx, request.id).await?;
    let id = action.id;
    let component_id = action.component(&ctx).await?.id();
    let action_kind = action.prototype(&ctx).await?.kind;

    action.delete(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "remove_action",
        serde_json::json!({
            "how": "/change_set/remove_action",
            "action_id": id.clone(),
            "action_kind": action_kind,
            "component_id": component_id.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    WsEvent::action_removed(&ctx, component_id, id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
