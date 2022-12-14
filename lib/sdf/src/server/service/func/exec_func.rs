use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    job::definition::DependentValuesUpdate, AttributePrototype, DalContext, Func, FuncBackendKind,
    FuncId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecFuncResponse {
    pub success: bool,
}

async fn update_values_for_func(ctx: &DalContext, func: &Func) -> FuncResult<()> {
    let prototypes = AttributePrototype::find_for_func(ctx, func.id()).await?;
    for proto in prototypes {
        for value in proto.attribute_values(ctx).await?.iter_mut() {
            value.update_from_prototype_function(ctx).await?;
            ctx.enqueue_job(DependentValuesUpdate::new(ctx, *value.id()))
                .await;
        }
    }

    Ok(())
}

pub async fn exec_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ExecFuncRequest>,
) -> FuncResult<Json<ExecFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    if func.backend_kind() == &FuncBackendKind::JsAttribute {
        update_values_for_func(&ctx, &func).await?;
    }

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(ExecFuncResponse { success: true }))
}
