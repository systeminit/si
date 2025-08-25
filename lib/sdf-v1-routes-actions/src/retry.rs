use axum::Json;
use dal::{
    Func,
    WsEvent,
    action::{
        Action,
        ActionId,
        ActionState,
        prototype::ActionPrototype,
    },
};
use sdf_extract::{
    HandlerContext,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use si_events::audit_log::AuditLogKind;

use super::{
    ActionError,
    ActionResult,
};

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
    println!("test");
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    for action_id in request.ids {
        let action = Action::get_by_id(&ctx, action_id).await?;

        let prototype_id = Action::prototype_id(&ctx, action_id).await?;
        let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;
        let func_id = ActionPrototype::func_id(&ctx, prototype_id).await?;
        let func = Func::get_by_id(&ctx, func_id).await?;
        ctx.write_audit_log(
            AuditLogKind::RetryAction {
                prototype_id,
                action_kind: prototype.kind.into(),
                func_id,
                func_display_name: func.display_name,
                func_name: func.name.clone(),
                component_id: None,
            },
            func.name,
        )
        .await?;

        match action.state() {
            ActionState::Running | ActionState::Dispatched => {
                return Err(ActionError::InvalidOnHoldTransition(action_id));
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
