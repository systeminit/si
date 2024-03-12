use axum::{extract::Query, Json};
use dal::code_view::CodeView;
use dal::{Component, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeResponse {
    pub code_views: Vec<CodeView>,
    pub has_code: bool,
}

pub async fn get_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetCodeRequest>,
) -> ComponentResult<Json<GetCodeResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let (code_views, has_code) = Component::list_code_generated(&ctx, request.component_id).await?;

    Ok(Json(GetCodeResponse {
        code_views,
        has_code,
    }))
}
