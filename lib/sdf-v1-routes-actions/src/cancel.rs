use axum::Json;
use dal::{
    Func,
    WsEvent,
    action::{
        Action,
        ActionId,
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

use super::ActionResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PutOnHoldRequest {
    pub ids: Vec<ActionId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}
// batched
pub async fn cancel(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<PutOnHoldRequest>,
) -> ActionResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    for action_id in request.ids {
        let prototype_id = Action::prototype_id(&ctx, action_id).await?;
        let prototype = ActionPrototype::get_by_id(&ctx, prototype_id).await?;
        let func_id = ActionPrototype::func_id(&ctx, prototype_id).await?;
        let func = Func::get_by_id(&ctx, func_id).await?;
        ctx.write_audit_log(
            AuditLogKind::CancelAction {
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

        Action::remove_by_id(&ctx, action_id).await?;
    }
    WsEvent::action_list_updated(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(())
}
