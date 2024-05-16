use axum::Json;
use dal::action::{Action, ActionState};
use dal::{ActionId, Visibility, WsEvent};
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

pub async fn put_on_hold(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<PutOnHoldRequest>,
) -> ActionResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    for action_id in request.ids {
        let action = Action::get_by_id(&ctx, action_id).await?;

        match action.state() {
            ActionState::Running | ActionState::Dispatched | ActionState::OnHold => {
                return Err(ActionError::InvalidOnHoldTransition(action_id))
            }
            ActionState::Queued | ActionState::Failed => {}
        }

        Action::set_state(&ctx, action.id(), ActionState::OnHold).await?;
    }
    WsEvent::action_list_updated(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}
