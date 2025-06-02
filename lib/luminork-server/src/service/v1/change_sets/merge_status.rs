use axum::response::Json;
use dal::{
    ChangeSetId,
    ComponentId,
    DalContext,
    action::{
        Action,
        ActionState,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
};
use serde::Serialize;
use serde_json::json;
use si_events::{
    ActionId,
    ChangeSetStatus,
};
use utoipa::ToSchema;

use super::ChangeSetResult;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ChangeSetError,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier")
    ),
    tag = "change_sets",
    summary = "Get Change Set post merge status",
    responses(
        (status = 200, description = "Change Set merge status retrieved successfully", body = MergeStatusV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn merge_status(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> ChangeSetResult<Json<MergeStatusV1Response>> {
    let change_set = ctx.change_set()?.into_frontend_type(ctx).await?;

    tracker.track(
        ctx,
        "api_change_set_merge_status",
        json!({
            "change_set_name": change_set.name,
            "change_set_status": change_set.status
        }),
    );

    let actions = match change_set.status {
        // Grab action status from HEAD since we don't get updates anymore after being applied
        ChangeSetStatus::Applied => {
            get_action_statuses(&ctx.clone_with_base().await?, change_set.id).await?
        }
        _ => get_action_statuses(ctx, change_set.id).await?,
    };

    Ok(Json(MergeStatusV1Response {
        change_set,
        actions,
    }))
}

/// Helper to get action statuses for a change set
async fn get_action_statuses(
    ctx: &DalContext,
    change_set_id: ChangeSetId,
) -> Result<Vec<MergeStatusV1ResponseAction>, ChangeSetError> {
    let mut actions = Vec::new();

    for action_id in Action::all_ids(ctx).await? {
        let action = Action::get_by_id(ctx, action_id).await?;
        let ActionPrototype { kind, name, .. } = Action::prototype(ctx, action_id).await?;
        let component = match Action::component(ctx, action_id).await? {
            Some(component) => Some(MergeStatusV1ResponseActionComponent {
                id: component.id(),
                name: component.name(ctx).await?,
            }),
            None => None,
        };

        if action.originating_changeset_id() == change_set_id {
            actions.push(MergeStatusV1ResponseAction {
                id: action_id,
                component,
                state: action.state(),
                kind,
                name,
            })
        }
    }

    Ok(actions)
}

/// Response for merge status
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "changeSet": {
        "id": "01FXNV4P306V3KGZ73YSVN8A60",
        "name": "My feature",
        "status": "Ready"
    },
    "actions": [
        {
            "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
            "component": {
                "id": "01H9ZQD35JPMBGHH69BT0Q79AB",
                "name": "my-ec2-instance"
            },
            "state": "Pending",
            "kind": "Create",
            "name": "Create EC2 Instance"
        }
    ]
}))]
pub struct MergeStatusV1Response {
    #[schema(value_type = Object, example = json!({"id": "01FXNV4P306V3KGZ73YSVN8A60", "name": "My feature", "status": "Ready"}))]
    pub change_set: si_frontend_types::ChangeSet,
    pub actions: Vec<MergeStatusV1ResponseAction>,
}

/// Action item in merge status response
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
    "component": {
        "id": "01H9ZQD35JPMBGHH69BT0Q79AB",
        "name": "my-ec2-instance"
    },
    "state": "Pending",
    "kind": "Create",
    "name": "Create EC2 Instance"
}))]
pub struct MergeStatusV1ResponseAction {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub id: ActionId,
    pub component: Option<MergeStatusV1ResponseActionComponent>,
    #[schema(value_type = String, example = "Pending")]
    pub state: ActionState,
    #[schema(value_type = String, example = "Create")]
    pub kind: ActionKind,
    #[schema(example = "Create EC2 Instance")]
    pub name: String,
}

/// Component details in action response
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "01H9ZQD35JPMBGHH69BT0Q79AB",
    "name": "my-ec2-instance"
}))]
pub struct MergeStatusV1ResponseActionComponent {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79AB")]
    pub id: ComponentId,
    #[schema(example = "my-ec2-instance")]
    pub name: String,
}
