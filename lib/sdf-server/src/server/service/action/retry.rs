use axum::Json;
use dal::action::{Action, ActionState};
use dal::{action::ActionId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::ActionResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::action::ActionError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetryRequest {
    ids: Vec<ActionId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn retry(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RetryRequest>,
) -> ActionResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    for action_id in request.ids {
        let action = Action::get_by_id(&ctx, action_id).await?;

        match action.state() {
            ActionState::Running | ActionState::Dispatched => {
                return Err(ActionError::InvalidOnHoldTransition(action_id))
            }
            ActionState::Queued | ActionState::Failed | ActionState::OnHold => {}
        }
        Action::set_state(&ctx, action.id(), ActionState::Queued).await?;
    }
    WsEvent::action_list_updated(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(())
}
