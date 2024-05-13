use axum::extract::Query;
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

pub async fn put_on_hold(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<PutOnHoldRequest>,
) -> ActionResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let action = Action::get_by_id(&ctx, request.id).await?;

    match action.state() {
        ActionState::Running | ActionState::Dispatched | ActionState::OnHold => {
            return Err(ActionError::InvalidOnHoldTransition(request.id))
        }
        ActionState::Queued | ActionState::Failed => {}
    }

    Action::remove_by_id(&ctx, action.id()).await?;

    Ok(())
}
