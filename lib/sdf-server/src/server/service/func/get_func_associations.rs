use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use dal::func::FuncAssociations;
use dal::{Func, FuncId, Visibility};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncAssociationsRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncAssociationsResponse {
    pub associations: Option<FuncAssociations>,
}

pub async fn get_func_associations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<GetFuncAssociationsRequest>,
) -> FuncResult<Json<GetFuncAssociationsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id_or_error(&ctx, request.id).await?;
    let (associations, _input_type) = FuncAssociations::from_func(&ctx, &func).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_func_associations",
        serde_json::json!({
            "how": "/func/get_func_associations",
            "func_id": func.id,
            "func_name": func.name
        }),
    );

    Ok(Json(GetFuncAssociationsResponse { associations }))
}
