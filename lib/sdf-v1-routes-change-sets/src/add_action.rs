use axum::extract::{
    Host,
    Json,
    OriginalUri,
};
use dal::{
    ActionPrototypeId,
    ChangeSet,
    Component,
    ComponentId,
    Func,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
};
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use si_events::audit_log::AuditLogKind;

use super::ChangeSetResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddActionRequest {
    pub prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn add_action(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<AddActionRequest>,
) -> ChangeSetResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let prototype = ActionPrototype::get_by_id(&ctx, request.prototype_id).await?;

    match prototype.kind {
        ActionKind::Create | ActionKind::Destroy | ActionKind::Update | ActionKind::Refresh => {
            let maybe_duplicate_action =
                Action::find_for_kind_and_component_id(&ctx, request.component_id, prototype.kind)
                    .await?;
            if !maybe_duplicate_action.is_empty() {
                return Err(super::ChangeSetError::ActionAlreadyEnqueued(prototype.id));
            }
        }

        dal::action::prototype::ActionKind::Manual => {}
    }

    let component = Component::get_by_id(&ctx, request.component_id).await?;
    let action = Action::new(&ctx, request.prototype_id, Some(request.component_id)).await?;
    let func_id = ActionPrototype::func_id(&ctx, prototype.id).await?;
    let func = Func::get_by_id(&ctx, func_id).await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_action_v2",
        serde_json::json!({
            "how": "/change_set/add_action",
            "action_id": action.id(),
            "action_kind": prototype.kind,
            "component_id": component.id(),
            "change_set_id": ctx.change_set_id(),
        }),
    );
    // todo add ws event here
    ctx.write_audit_log(
        AuditLogKind::AddAction {
            prototype_id: prototype.id(),
            action_kind: prototype.kind.into(),
            func_id,
            func_display_name: func.display_name,
            func_name: func.name.clone(),
            component_id: Some(component.id()),
        },
        func.name,
    )
    .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
