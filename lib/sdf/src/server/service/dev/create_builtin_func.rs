use axum::Json;

use dal::{FuncBackendKind, HistoryActor, RequestContext, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DevResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::func::create_func::CreateFuncResponse;
use crate::service::func::FuncError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateBuiltinFuncRequest {
    pub kind: FuncBackendKind,
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn create_builtin_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateBuiltinFuncRequest>,
) -> DevResult<Json<CreateFuncResponse>> {
    let mut ctx = builder
        .build(RequestContext::new_universal_head(HistoryActor::SystemInit))
        .await?;

    // TODO(nick): fix this module.
    #[allow(clippy::match_single_binding)]
    let func = match request.kind {
        _ => Err(FuncError::FuncNotSupported)?,
    };

    // TODO(nick): restore the ability to author qualification builtins.
    // let mut func = Func::new(
    //     &ctx,
    //     request.name,
    //     FuncBackendKind::JsAttribute,
    //     FuncBackendResponseType::Qualification,
    // )
    //     .await?;
    // func.set_code_plaintext(&ctx, Some(DEFAULT_QUALIFICATION_CODE))
    //     .await?;
    // func.set_handler(&ctx, Some("qualification".to_owned()))
    //     .await?;
    // func

    dal::builtins::func_persist(&ctx, &func).await?;

    // Update the ctx with the account details for proper WS signaling
    ctx.update_from_request_context(request_ctx.build(request.visibility));

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        variant: (&func).try_into()?,
        name: func.name().to_owned(),
        code: func.code_plaintext()?,
    }))
}
