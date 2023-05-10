use axum::extract::Query;
use axum::Json;

use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{Component, ComponentId, ComponentView, StandardModel, Visibility};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InspectComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

type InspectComponentResponse = ComponentView;

pub async fn inspect_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<InspectComponentRequest>,
) -> ComponentResult<Json<InspectComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let is_component_in_tenancy = Component::is_in_tenancy(&ctx, request.component_id).await?;
    let is_component_in_visibility = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .is_some();
    if is_component_in_tenancy && !is_component_in_visibility {
        return Err(ComponentError::InvalidVisibility);
    }

    let view = ComponentView::new(&ctx, request.component_id).await?;

    Ok(Json(view))
}
