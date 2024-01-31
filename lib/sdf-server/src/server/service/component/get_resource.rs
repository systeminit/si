use axum::{extract::Query, Json};
use dal::{ComponentId, ResourceView, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetResourceRequest>,
) -> ComponentResult<Json<GetResourceResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let resource = ResourceView::get_by_component_id(&ctx, &request.component_id).await?;
    Ok(Json(GetResourceResponse { resource }))
}
