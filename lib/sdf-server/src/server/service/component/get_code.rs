use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::code_view::CodeView;
use dal::{Component, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

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
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetCodeRequest>,
) -> ComponentResult<Json<GetCodeResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let (code_views, has_code) = Component::list_code_generated(&ctx, request.component_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_code",
        serde_json::json!({
            "how": "/component/get_code",
            "component_id": request.component_id.clone(),
            "code_views": code_views.clone().len(),
            "has_code": has_code.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(GetCodeResponse {
        code_views,
        has_code,
    }))
}
