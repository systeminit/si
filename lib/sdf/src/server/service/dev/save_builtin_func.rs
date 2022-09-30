use axum::Json;
use serde::{Deserialize, Serialize};

use dal::{
    Func, FuncBackendKind, FuncId, HistoryActor, RequestContext, StandardModel, Visibility, WsEvent,
};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::dev::DevError;
use crate::service::func::FuncAssociations;

use super::DevResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveBuiltinFuncRequest {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub associations: Option<FuncAssociations>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveBuiltinFuncResponse {
    pub success: bool,
}

pub async fn save_builtin_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveBuiltinFuncRequest>,
) -> DevResult<Json<SaveBuiltinFuncResponse>> {
    let mut ctx = builder
        .build(RequestContext::new_universal_head(HistoryActor::SystemInit))
        .await?;

    let mut func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(DevError::FuncNotFound)?;

    func.set_display_name(&ctx, Some(request.name)).await?;
    func.set_description(&ctx, request.description).await?;
    func.set_handler(&ctx, request.handler).await?;
    func.set_backend_kind(&ctx, request.kind).await?;
    func.set_code_plaintext(&ctx, request.code.as_deref())
        .await?;

    dal::builtins::func_persist(&ctx, &func).await?;

    // Update the ctx with the account details for proper WS signaling
    ctx.update_from_request_context(request_ctx.build(request.visibility));

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(SaveBuiltinFuncResponse { success: true }))
}
