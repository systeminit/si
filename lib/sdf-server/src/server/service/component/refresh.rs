use axum::extract::OriginalUri;
use axum::Json;

use dal::{
    job::definition::RefreshJob, Component, ComponentId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
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

    let component_id = request.component_id;
    let result = ctx
        .run_with_deleted_visibility(|ctx| async move {
            // If a component does not exist on head, we should just consider it "refreshed" right away.
            if let Some(component_id) = component_id {
                let bailout =
                    if let Some(component) = Component::get_by_id(&ctx, &component_id).await? {
                        component.is_destroyed()
                    } else {
                        false
                    };

                if bailout {
                    WsEvent::resource_refreshed(&ctx, component_id)
                        .await?
                        .publish_on_commit(&ctx)
                        .await?;
                    ctx.commit().await?;

                    return Ok::<_, ComponentError>(Some(Json(RefreshResponse { success: true })));
                }
            }
            Ok(None)
        })
        .await?;

    if let Some(result) = result {
        return Ok(result);
    }

    let component_ids = if let Some(component_id) = request.component_id {
        vec![component_id]
    } else {
        ctx.run_with_deleted_visibility(|ctx| async move {
            let component_ids = Component::list(&ctx)
                .await?
                .into_iter()
                .filter(|c| c.visibility().deleted_at.is_none() || c.needs_destroy())
                .map(|c| *c.id())
                .collect();
            Ok::<_, ComponentError>(component_ids)
        })
        .await?
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

    ctx.enqueue_job(RefreshJob::new(
        ctx.access_builder(),
        *ctx.visibility(),
        component_ids,
    ))
    .await?;

    ctx.commit().await?;

    Ok(Json(RefreshResponse { success: true }))
}
