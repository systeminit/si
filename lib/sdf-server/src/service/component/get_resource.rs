use axum::{
    extract::{Host, OriginalUri, Query},
    Json,
};
use dal::{component::resource::ResourceView, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceResponse {
    pub resource: ResourceView,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn get_resource(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetResourceRequest>,
) -> ComponentResult<Json<GetResourceResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let resource = ResourceView::get_by_component_id(&ctx, request.component_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "get_resource",
        serde_json::json!({
            "how": "/component/get_resource",
            "component_id": request.component_id.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(GetResourceResponse { resource }))
}
