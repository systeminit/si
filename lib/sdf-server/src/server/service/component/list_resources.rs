use axum::extract::Query;
use axum::Json;
use dal::{ComponentId, ResourceView, Visibility};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ListResourcesResponse = HashMap<ComponentId, ResourceView>;

pub async fn list_resources(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListResourcesRequest>,
) -> ComponentResult<Json<ListResourcesResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let resources = ResourceView::list_with_deleted(&ctx).await?;
    Ok(Json(resources))
}
