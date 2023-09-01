use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::server::service::change_set::ChangeSetError;
use axum::Json;
use dal::{Action, ActionId, StandardModel, Visibility, WsEvent};
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

    let mut action = Action::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(ChangeSetError::ActionNotFound(request.id))?;
    action.delete_by_id(&ctx).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
