use axum::{
    Json,
    extract::Path,
};
use dal::{
    Func,
    WsEvent,
    action::{
        Action,
        prototype::ActionPrototype,
    },
};
use serde::Serialize;
use serde_json::json;
use si_events::{
    ActionState,
    audit_log::AuditLogKind,
};
use utoipa::ToSchema;

use super::{
    ActionV1RequestPath,
    ActionsError,
    ActionsResult,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/retry",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("action_id" = String, Path, description = "Action identifier"),
    ),
    tag = "actions",
    summary = "Retry action",
    responses(
        (status = 200, description = "Action successfully requeued", body = RetryActionV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Action not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn retry_action(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ActionV1RequestPath { action_id }): Path<ActionV1RequestPath>,
) -> ActionsResult<Json<RetryActionV1Response>> {
    let action = Action::get_by_id(ctx, action_id)
        .await
        .map_err(|_a| ActionsError::ActionNotFound(action_id))?;

    let prototype_id = Action::prototype_id(ctx, action_id).await?;
    let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
    let func_id = ActionPrototype::func_id(ctx, prototype_id).await?;
    let func = Func::get_by_id(ctx, func_id).await?;

    ctx.write_audit_log(
        AuditLogKind::RetryAction {
            prototype_id,
            action_kind: prototype.kind.into(),
            func_id,
            func_display_name: func.display_name.clone(),
            func_name: func.name.clone(),
        },
        func.name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_retry_action",
        json!({
            "action_id": action_id,
            "func_display_name": func.display_name.unwrap_or("unknown".to_string()),
            "func_name": func.name.clone()
        }),
    );

    match action.state() {
        ActionState::Running | ActionState::Dispatched => {
            return Err(ActionsError::InvalidOnHoldTransition(action_id));
        }
        ActionState::Queued | ActionState::Failed | ActionState::OnHold => {}
    }
    Action::set_state(ctx, action.id(), ActionState::Queued).await?;

    WsEvent::action_list_updated(ctx)
        .await?
        .publish_on_commit(ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(RetryActionV1Response { success: true }))
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetryActionV1Response {
    #[schema(value_type = bool)]
    pub success: bool,
}
