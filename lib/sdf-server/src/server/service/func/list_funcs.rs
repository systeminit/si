use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{Func, FuncBackendKind, FuncId, FuncVariant, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListedFuncView {
    pub id: FuncId,
    pub handler: Option<String>,
    pub variant: FuncVariant,
    pub name: String,
    pub display_name: Option<String>,
    pub is_builtin: bool,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
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

    let try_func_views: Vec<Result<ListedFuncView, FuncError>> = Func::find_by_attr_in(
        &ctx,
        "backend_kind",
        &[
            &FuncBackendKind::JsAction.as_ref().to_string(),
            &FuncBackendKind::JsAuthentication.as_ref().to_string(),
            &FuncBackendKind::JsAttribute.as_ref().to_string(),
            &FuncBackendKind::JsValidation.as_ref().to_string(),
        ],
    )
    .await?
    .iter()
    .filter(|f| !f.hidden())
    .map(|func| {
        Ok(ListedFuncView {
            id: func.id().to_owned(),
            handler: func.handler().map(|handler| handler.to_owned()),
            variant: func.try_into()?,
            name: func.name().into(),
            display_name: func.display_name().map(Into::into),
            is_builtin: func.builtin(),
        })
    })
    .collect();

    let mut funcs = vec![];
    for func_view in try_func_views {
        match func_view {
            Ok(func_view) => funcs.push(func_view),
            Err(err) => Err(err)?,
        }
    }

    Ok(Json(ListFuncsResponse { funcs }))
}
