use axum::extract::Query;
use axum::Json;
use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::{Action, ActionState};
use dal::deprecated_action::runner::ActionHistoryView;
use dal::{
    ActionCompletionStatus, ActionId, ActionPrototypeId, ChangeSetId, DeprecatedActionBatch,
    DeprecatedActionBatchId, Visibility,
};
use serde::{Deserialize, Serialize};

use super::ActionResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub id: ActionId,
    pub prototype_id: ActionPrototypeId,
    pub name: String,
    pub description: Option<String>,
    pub kind: ActionKind,
    pub state: ActionState,
    pub originating_changeset_id: ChangeSetId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoadQueuedRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type LoadQueuedResponse = Vec<ActionView>;

pub async fn load_queued(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<LoadQueuedRequest>,
) -> ActionResult<Json<LoadQueuedResponse>> {
    let ctx = builder.build_head(request_ctx).await?;

    let action_ids = Action::list_topologically(&ctx).await?;

    let mut queued = Vec::new();

    for action_id in action_ids {
        let action = Action::get_by_id(&ctx, action_id).await?;
        if action.state() == ActionState::Queued {
            let prototype_id = Action::action_prototype_id(&ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;

            let action_view = ActionView {
                id: action_id,
                prototype_id: prototype.id(),
                name: prototype.name().clone(),
                description: prototype.description().clone(),
                kind: prototype.kind(),
                state: action.state(),
                originating_changeset_id: action.originating_changeset_id(),
            };
            queued.push(action_view);
        }
    }

    Ok(Json(queued))
}
