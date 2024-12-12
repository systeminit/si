use axum::{
    extract::{Host, OriginalUri, Query},
    Json,
};
use dal::{component::debug::ComponentDebugView, ComponentId, Visibility};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    routes::AppError,
    track,
};

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
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    Query(request): Query<DebugComponentRequest>,
) -> Result<Json<ComponentDebugView>, AppError> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let debug_view = ComponentDebugView::new(&ctx, request.component_id).await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "debug",
        serde_json::json!({
            "how": "/component/debug",
            "component_id": request.component_id.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(debug_view))
}
