use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use dal::func::view::FuncView;
use dal::{Func, FuncId, Visibility};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetLatestFuncExecutionRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

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
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<GetFuncRequest>,
) -> FuncResult<Json<GetFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id_or_error(&ctx, request.id).await?;
    let view = FuncView::assemble(&ctx, &func).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_func",
        serde_json::json!({
            "how": "/func/get_func",
            "func_id": request.id,
            "func_name": func.name,
        }),
    );

    Ok(Json(view))
}
