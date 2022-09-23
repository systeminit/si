use super::{FuncArgumentView, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{func::argument::FuncArgument, FuncId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListArgumentsRequest {
    pub func_id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListArgumentsResponse {
    pub arguments: Vec<FuncArgumentView>,
}

pub async fn list_arguments(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListArgumentsRequest>,
) -> FuncResult<Json<ListArgumentsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let arguments = FuncArgument::list_for_func(&ctx, request.func_id)
        .await?
        .iter()
        .map(|func_arg| FuncArgumentView {
            id: *func_arg.id(),
            name: func_arg.name().to_owned(),
            kind: func_arg.kind().to_owned(),
            element_kind: func_arg.element_kind().cloned(),
        })
        .collect();

    Ok(Json(ListArgumentsResponse { arguments }))
}
