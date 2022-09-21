use axum::Json;
use dal::qualification_prototype::QualificationPrototypeContext;
use dal::{
    Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, QualificationPrototype,
    RequestContext, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::DevResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::func::create_func::{CreateFuncResponse, DEFAULT_QUALIFICATION_CODE};
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

    let func = match request.kind {
        FuncBackendKind::JsQualification => {
            let mut func = Func::new(
                &ctx,
                request.name,
                FuncBackendKind::JsQualification,
                FuncBackendResponseType::Qualification,
            )
            .await?;

            func.set_code_plaintext(&ctx, Some(DEFAULT_QUALIFICATION_CODE))
                .await?;
            func.set_handler(&ctx, Some("qualification".to_owned()))
                .await?;

            let _ =
                QualificationPrototype::new(&ctx, *func.id(), QualificationPrototypeContext::new())
                    .await?;

            func
        }
        _ => Err(FuncError::FuncNotSupported)?,
    };

    dal::builtins::func_persist(&func).await?;

    // Update the ctx with the account details for proper WS signaling
    ctx.update_from_request_context(request_ctx.build(request.visibility));

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func.name().to_owned(),
        code: func.code_plaintext()?,
        schema_variants: vec![],
    }))
}
