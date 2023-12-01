use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::{Json, OriginalUri};
use dal::{Action, ActionPrototypeId, ComponentId, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddActionRequest {
    pub prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn add_action(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<AddActionRequest>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let action = Action::new(&ctx, request.prototype_id, request.component_id).await?;
    let prototype = action.prototype(&ctx).await?;
    let component = action.component(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_action",
        serde_json::json!({
            "how": "/change_set/add_action",
            "prototype_id": prototype.id(),
            "prototype_kind": prototype.kind(),
            "component_name": component.name(&ctx).await?,
            "component_id": component.id(),
            "change_set_pk": ctx.visibility().change_set_pk,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
