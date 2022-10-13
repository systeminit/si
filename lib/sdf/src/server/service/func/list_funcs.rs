use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{Func, FuncBackendKind, FuncId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListedFuncView {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub is_builtin: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncsResponse {
    pub funcs: Vec<ListedFuncView>,
}

pub async fn list_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListFuncsRequest>,
) -> FuncResult<Json<ListFuncsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let funcs = Func::find_by_attr_in(
        &ctx,
        "backend_kind",
        &[
            &FuncBackendKind::JsQualification.as_ref().to_string(),
            &FuncBackendKind::JsAttribute.as_ref().to_string(),
            &FuncBackendKind::JsCodeGeneration.as_ref().to_string(),
            &FuncBackendKind::JsCommand.as_ref().to_string(),
            &FuncBackendKind::JsConfirmation.as_ref().to_string(),
        ],
    )
    .await?
    .iter()
    .map(|func| ListedFuncView {
        id: func.id().to_owned(),
        handler: func.handler().map(|handler| handler.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func
            .display_name()
            .unwrap_or_else(|| func.name())
            .to_owned(),
        is_builtin: func.is_builtin(),
    })
    .collect();

    Ok(Json(ListFuncsResponse { funcs }))
}
