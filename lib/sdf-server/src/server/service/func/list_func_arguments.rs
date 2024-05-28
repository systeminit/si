use axum::{extract::Query, Json};
use dal::func::argument::FuncArgument;
use dal::{FuncId, Visibility};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncArgumentsRequest {
    pub func_id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncArgumentsResponse {
    pub func_arguments: Vec<FuncArgument>,
}

pub async fn list_func_arguments(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListFuncArgumentsRequest>,
) -> FuncResult<Json<ListFuncArgumentsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func_arguments = FuncArgument::list_for_func(&ctx, request.func_id).await?;

    Ok(Json(ListFuncArgumentsResponse { func_arguments }))
}
