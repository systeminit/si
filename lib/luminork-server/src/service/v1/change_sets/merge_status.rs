use axum::response::Json;
use dal::{
    action::{
        prototype::{ActionKind, ActionPrototype},
        Action, ActionState,
    },
    ChangeSetId, ComponentId, DalContext,
};
use serde::Serialize;
use si_events::{ActionId, ChangeSetStatus};
use utoipa::ToSchema;

use crate::extract::change_set::ChangeSetDalContext;

use crate::service::v1::ChangeSetError;

/// Get status of a change set and its actions
#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier")
    ),
    tag = "change_sets",
    responses(
        (status = 200, description = "Change set merge status retrieved successfully", body = MergeStatusV1Response),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn merge_status(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
) -> Result<Json<MergeStatusV1Response>, ChangeSetError> {
    let change_set = ctx.change_set()?.into_frontend_type(ctx).await?;

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
pub struct MergeStatusV1Response {
    #[schema(value_type = Object, example = json!({"id": "01FXNV4P306V3KGZ73YSVN8A60", "name": "My feature", "status": "Ready"}))]
    pub change_set: si_frontend_types::ChangeSet,
    pub actions: Vec<MergeStatusV1ResponseAction>,
}

/// Action item in merge status response
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MergeStatusV1ResponseAction {
    #[schema(value_type = String)]
    pub id: ActionId,
    pub component: Option<MergeStatusV1ResponseActionComponent>,
    #[schema(value_type = String)]
    pub state: ActionState,
    #[schema(value_type = String)]
    pub kind: ActionKind,
    pub name: String,
}

/// Component details in action response
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MergeStatusV1ResponseActionComponent {
    #[schema(value_type = String)]
    pub id: ComponentId,
    pub name: String,
}
