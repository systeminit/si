use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::component::diff::ComponentDiff;
use dal::{Component, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiffRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiffResponse {
    pub component_diff: ComponentDiff,
}

pub async fn get_diff(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiffRequest>,
) -> ComponentResult<Json<GetDiffResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component_diff = Component::get_diff(&ctx, request.component_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_diff",
        serde_json::json!({
            "how": "/component/get_diff",
            "component_id": request.component_id.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(GetDiffResponse { component_diff }))
}
