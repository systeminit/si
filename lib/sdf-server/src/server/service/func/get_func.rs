use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use dal::func::view::FuncView;
use dal::{Func, FuncId, Visibility};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetLatestFuncExecutionRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

// #[derive(Deserialize, Serialize, Debug)]
// #[serde(rename_all = "camelCase")]
// pub struct GetLatestFuncExecutionResponse {
//     pub id: FuncId,
//     pub state: FuncExecutionState,
//     pub value: Option<serde_json::Value>,
//     pub output_stream: Option<Vec<OutputStream>>,
//     pub function_failure: Option<FunctionResultFailure>,
// }

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

type GetFuncResponse = FuncView;

pub async fn get_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetFuncRequest>,
) -> FuncResult<Json<GetFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id_or_error(&ctx, request.id).await?;
    let view = FuncView::assemble(&ctx, &func).await?;

    Ok(Json(view))
}

// pub async fn get_latest_func_execution(
//     HandlerContext(builder): HandlerContext,
//     AccessBuilder(request_ctx): AccessBuilder,
//     Query(request): Query<GetLatestFuncExecutionRequest>,
// ) -> FuncResult<Json<GetLatestFuncExecutionResponse>> {
//     let ctx = builder.build(request_ctx.build(request.visibility)).await?;

//     let func_execution_result =
//         FuncExecution::get_latest_execution_by_func_id(&ctx, &request.id).await?;

//     Ok(Json(GetLatestFuncExecutionResponse {
//         id: *func_execution_result.func_id(),
//         state: func_execution_result.state(),
//         value: func_execution_result.value().cloned(),
//         output_stream: func_execution_result.output_stream().cloned(),
//         function_failure: func_execution_result.function_failure().clone(),
//     }))
// }
