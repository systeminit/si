use axum::Json;
use dal::func::argument::FuncArgument;
use dal::{AttributePrototype, Func, FuncBackendKind, FuncId, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RevertFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RevertFuncResponse {
    pub success: bool,
}

pub async fn revert_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RevertFuncRequest>,
) -> FuncResult<Json<RevertFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let is_revertible = super::is_func_revertible(&ctx, &func).await?;

    if !is_revertible {
        Err(FuncError::FuncNotRevertible)?
    } else {
        if func.backend_kind() == &FuncBackendKind::JsAttribute {
            for proto in AttributePrototype::find_for_func(&ctx, func.id()).await? {
                if proto.visibility().in_change_set() {
                    AttributePrototype::hard_delete_if_in_changeset(&ctx, proto.id()).await?;
                }
            }
        }

        for arg in FuncArgument::list_for_func(&ctx, *func.id()).await? {
            if arg.visibility().in_change_set() {
                arg.hard_delete(&ctx).await?;
            }
        }

        func.hard_delete(&ctx).await?;

        WsEvent::change_set_written(&ctx)
            .await?
            .publish_on_commit(&ctx)
            .await?;

        ctx.commit().await?;

        Ok(Json(RevertFuncResponse { success: true }))
    }
}
