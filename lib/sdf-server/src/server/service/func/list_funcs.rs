use std::time::Instant;

use super::{FuncError, FuncResult, FuncVariant};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{Func, FuncBackendKind, FuncId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

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
    info!("==================");
    let start = tokio::time::Instant::now();

    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    info!("after context build: {:?}", start.elapsed());

    //ctx.workspace_snapshot()?.lock().await.dot();

    let funcs = ctx
        .workspace_snapshot()?
        .lock()
        .await
        .list_funcs(&ctx)
        .await?;

    info!("after content store fetch: {:?}", start.elapsed());

    let customizable_backend_kinds = [
        FuncBackendKind::JsAction,
        FuncBackendKind::JsAttribute,
        FuncBackendKind::JsValidation,
    ];

    let try_func_views: Vec<FuncResult<ListedFuncView>> = funcs
        .iter()
        .filter(|f| {
            if f.hidden {
                return false;
            } else {
                return customizable_backend_kinds.contains(&f.backend_kind);
            }
        })
        .map(|func| {
            Ok(ListedFuncView {
                id: func.id,
                handler: func.handler.to_owned().map(|handler| handler.to_owned()),
                variant: func.try_into()?,
                name: func.name.to_owned(),
                display_name: func.display_name.to_owned().map(Into::into),
                is_builtin: func.builtin,
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

    funcs.sort_by(|a, b| a.name.cmp(&b.name));

    info!("after list funcs: {:?}", start.elapsed());

    Ok(Json(ListFuncsResponse { funcs }))
}
