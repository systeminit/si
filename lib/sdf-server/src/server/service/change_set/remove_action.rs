use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
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
    Json(request): Json<RemoveActionRequest>,
) -> ChangeSetResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let action = DeprecatedAction::get_by_id(&ctx, request.id).await?;
    let id = action.id;
    let component_id = action.component(&ctx).await?.id();

    action.delete(&ctx).await?;

    WsEvent::action_removed(&ctx, component_id, id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
