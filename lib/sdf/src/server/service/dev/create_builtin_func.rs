use axum::Json;
use dal::qualification_prototype::QualificationPrototypeContext;
use dal::{
    Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, QualificationPrototype,
    ReadTenancy, StandardModel, Visibility, WriteTenancy, WsEvent,
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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateBuiltinFuncRequest>,
) -> DevResult<Json<CreateFuncResponse>> {
    let txns = txns.start().await?;

    let universal_req_ctx = dal::context::AccessBuilder::new(
        ReadTenancy::new_universal(),
        WriteTenancy::new_universal(),
        HistoryActor::SystemInit,
    );
    let ctx = builder.build(universal_req_ctx.build(Visibility::new_head(false)), &txns);

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

    dal::builtins::func::persist(&func).await?;

    let acct_ctx = builder.build(request_ctx.build(request.visibility), &txns);
    WsEvent::change_set_written(&acct_ctx)
        .publish(&acct_ctx)
        .await?;

    txns.commit().await?;

    Ok(Json(CreateFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func.name().to_owned(),
        code: func.code_plaintext()?,
        schema_variants: vec![],
    }))
}
