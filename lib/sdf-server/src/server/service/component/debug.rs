use axum::extract::{OriginalUri, Query};
use axum::Json;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
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
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    Query(request): Query<DebugComponentRequest>,
) -> ComponentResult<Json<ComponentDebugView>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let debug_view = ComponentDebugView::new(&ctx, request.component_id).await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "debug",
        serde_json::json!({
            "how": "/component/debug",
            "component_id": request.component_id.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(debug_view))
}
