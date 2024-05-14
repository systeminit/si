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
    pub ids: Vec<ActionId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}
// batched
pub async fn cancel(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<PutOnHoldRequest>,
) -> ActionResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    for action_id in request.ids {
        let action = Action::get_by_id(&ctx, action_id).await?;

        match action.state() {
            ActionState::Running | ActionState::Dispatched => {
                return Err(ActionError::InvalidActionCancellation(action_id))
            }
            ActionState::Failed | ActionState::OnHold | ActionState::Queued => {}
        }

        Action::remove_by_id(&ctx, action.id()).await?;
        // todo add wsevent here
    }

    ctx.commit().await?;

    Ok(())
}
