use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{Func, FuncBackendKind, FuncId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncResponse {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub code: Option<String>,
    pub is_builtin: bool,
}

pub async fn get_func(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetFuncRequest>,
) -> FuncResult<Json<GetFuncResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    txns.commit().await?;

    Ok(Json(GetFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func.name().to_owned(),
        code: func.code_plaintext()?,
        is_builtin: func.is_builtin(),
    }))
}
