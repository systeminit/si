use axum::{extract::Query, Json};
use dal::func::view::summary::FuncSummary;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncsResponse {
    pub funcs: Vec<FuncSummary>,
}

pub async fn list_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListFuncsRequest>,
) -> FuncResult<Json<ListFuncsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let funcs = FuncSummary::list(&ctx).await?;

    Ok(Json(ListFuncsResponse { funcs }))
}
