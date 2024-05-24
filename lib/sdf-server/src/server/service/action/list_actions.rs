use axum::extract::Query;
use axum::Json;
use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::{Action, ActionState};
use dal::{action::ActionId, ActionPrototypeId, ChangeSetId, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ActionResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub id: ActionId,
    pub prototype_id: ActionPrototypeId,
    pub component_id: Option<ComponentId>,
    pub name: String,
    pub description: Option<String>,
    pub kind: ActionKind,
    pub state: ActionState,
    pub originating_changeset_id: ChangeSetId,
    // Actions that will wait until I've successfully completed before running
    pub my_dependencies: Vec<ActionId>,
    // Things that need to finish before I can start
    pub dependent_on: Vec<ActionId>,
    // includes action ids that impact this status
    // this occurs when ancestors of this action are on hold or have failed
    pub hold_status_influenced_by: Vec<ActionId>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoadQueuedRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type LoadQueuedResponse = Vec<ActionView>;

pub async fn list_actions(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<LoadQueuedRequest>,
) -> ActionResult<Json<LoadQueuedResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let action_ids = Action::list_topologically(&ctx).await?;

    let mut queued = Vec::new();

    for action_id in action_ids {
        let action = Action::get_by_id(&ctx, action_id).await?;
        let prototype_id = Action::prototype_id(&ctx, action_id).await?;
        let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;

        let action_view = ActionView {
            id: action_id,
            prototype_id: prototype.id(),
            name: prototype.name().clone(),
            component_id: Action::component_id(&ctx, action_id).await?,
            description: prototype.description().clone(),
            kind: prototype.kind,
            state: action.state(),
            originating_changeset_id: action.originating_changeset_id(),
            my_dependencies: action.get_all_dependencies(&ctx).await?,
            dependent_on: Action::get_dependent_actions_by_id(&ctx, action_id).await?,
            hold_status_influenced_by: action.get_hold_status_influenced_by(&ctx).await?,
        };
        queued.push(action_view);
    }

    Ok(Json(queued))
}
