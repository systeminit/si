use axum::{extract::Query, Json};
use dal::component::diff::ComponentDiff;
use dal::{ComponentId, ComponentView, ComponentViewProperties, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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
    pub json: ComponentViewProperties,
}

pub async fn json(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<JsonRequest>,
) -> ComponentResult<Json<JsonResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let curr_component_view = ComponentView::new(&ctx, request.component_id).await?;
    if curr_component_view.properties.is_null() {
        return Ok(Json(JsonResponse {
            json: ComponentViewProperties::default(),
        }));
    }

    let mut json = ComponentViewProperties::try_from(curr_component_view)?;

    Ok(Json(JsonResponse { json }))
}
