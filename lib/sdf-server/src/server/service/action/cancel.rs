use axum::Json;
use dal::action::{Action, ActionState};
use dal::{ActionId, Visibility};
use serde::{Deserialize, Serialize};

use super::ActionResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::action::ActionError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PutOnHoldRequest {
    id: ActionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn cancel(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<PutOnHoldRequest>,
) -> ActionResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let action = Action::get_by_id(&ctx, request.id).await?;

    match action.state() {
        ActionState::Running | ActionState::Dispatched => {
            return Err(ActionError::InvalidActionCancellation(request.id))
        }
        ActionState::Failed | ActionState::OnHold | ActionState::Queued => {}
    }

    Action::remove_by_id(&ctx, action.id()).await?;

    ctx.commit().await?;

    Ok(())
}
