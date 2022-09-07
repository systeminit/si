use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    Func, FuncBackendKind, FuncId, QualificationPrototype, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

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
    let ctx = builder
        .build(request_ctx.clone().build(request.visibility))
        .await?;

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let is_revertable = super::is_func_revertable(&ctx, &func).await?;

    if !is_revertable {
        Err(FuncError::FuncNotRevertable)?
    } else {
        match func.backend_kind() {
            FuncBackendKind::JsAttribute => {}
            FuncBackendKind::JsQualification => {
                for proto in QualificationPrototype::find_for_func(&ctx, func.id()).await? {
                    if proto.visibility().in_change_set() {
                        proto.hard_delete(&ctx).await?;
                    }
                }
            }
            _ => {}
        }

        func.hard_delete(&ctx).await?;

        WsEvent::change_set_written(&ctx).publish(&ctx).await?;

        ctx.commit().await?;

        Ok(Json(RevertFuncResponse { success: true }))
    }
}
