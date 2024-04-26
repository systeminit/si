use axum::extract::OriginalUri;
use axum::Json;

use dal::{job::definition::RefreshJob, Component, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub component_id: Option<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub success: bool,
}

pub async fn refresh(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RefreshRequest>,
) -> ComponentResult<Json<RefreshResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component_ids = if let Some(component_id) = request.component_id {
        vec![component_id]
    } else {
        let mut component_ids = Vec::new();
        for component in Component::list(&ctx).await? {
            if component.resource(&ctx).await?.payload.is_some() {
                component_ids.push(component.id());
            }
        }
        component_ids
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "refresh_resources",
        serde_json::json!({
            "component_ids": &component_ids,
        }),
    );

    // Parallelizes resource refreshing
    for component_id in component_ids {
        ctx.enqueue_refresh(RefreshJob::new(
            ctx.access_builder(),
            *ctx.visibility(),
            vec![component_id],
        ))
        .await?;
    }

    ctx.commit().await?;

    Ok(Json(RefreshResponse { success: true }))
}
