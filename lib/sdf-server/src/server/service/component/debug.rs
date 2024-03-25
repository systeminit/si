use axum::extract::Query;
use axum::Json;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{component::debug::ComponentDebugView, ComponentId, Visibility};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebugComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[instrument(level = "debug", skip_all)]
pub async fn debug_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<DebugComponentRequest>,
) -> ComponentResult<Json<ComponentDebugView>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let debug_view = ComponentDebugView::new(&ctx, request.component_id).await?;
    Ok(Json(debug_view))
}
