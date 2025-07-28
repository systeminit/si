use axum::Json;
use dal::{
    Func,
    action::{
        Action,
        prototype::ActionPrototype,
    },
};
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use super::ActionsResult;
use crate::{
    api_types::actions::v1::ActionViewV1,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/actions",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "actions",
    summary = "List queued actions",
    responses(
        (status = 200, description = "Actions retrieved successfully", body = GetActionsV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_actions(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> ActionsResult<Json<GetActionsV1Response>> {
    tracker.track(ctx, "api_get_actions", json!({}));

    let action_ids = Action::list_topologically(ctx).await?;

    let mut actions = Vec::new();
    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;

        let prototype_id = Action::prototype_id(ctx, action_id).await?;
        let func_id = ActionPrototype::func_id(ctx, prototype_id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;
        let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
        let func_run_id = ctx
            .layer_db()
            .func_run()
            .get_last_run_for_action_id_opt(ctx.events_tenancy().workspace_pk, action.id())
            .await?
            .map(|f| f.id());

        let action = ActionViewV1 {
            id: action_id,
            prototype_id: prototype.id(),
            name: prototype.name().clone(),
            component_id: Action::component_id(ctx, action_id).await?,
            description: func.display_name,
            kind: prototype.kind,
            state: action.state(),
            originating_change_set_id: action.originating_changeset_id(),
            func_run_id,
        };

        actions.push(action);
    }

    Ok(Json(GetActionsV1Response { actions }))
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "actions": [
        {
            "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
            "prototypeId": "01H9ZQD35JPMBGHH69BT0Q79AB",
            "componentId": "01H9ZQD35JPMBGHH69BT0Q79CD",
            "name": "Create EC2 Instance",
            "description": "Provisions a new EC2 instance in AWS",
            "kind": "Create",
            "state": "Pending",
            "originatingChangeSetId": "01H9ZQD35JPMBGHH69BT0Q79EF",
            "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79GH"
        }
    ]
}))]
pub struct GetActionsV1Response {
    pub actions: Vec<ActionViewV1>,
}
