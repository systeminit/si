use axum::{extract::Query, Json};
use dal::{component::properties::ComponentProperties, Component, ComponentId, Visibility};
use sdf_extract::{v1::AccessBuilder, HandlerContext};
use serde::{Deserialize, Serialize};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonResponse {
    pub json: ComponentProperties,
}

pub async fn json(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<JsonRequest>,
) -> ComponentResult<Json<JsonResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let json = Component::get_json_representation(&ctx, request.component_id).await?;

    Ok(Json(JsonResponse { json }))
}
