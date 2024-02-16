use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::{Component, ComponentId, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use dal::component::migrate::migrate_component_to_schema_variant;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MigrateToDefaultVariantRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn migrate_to_default_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Json(request): Json<MigrateToDefaultVariantRequest>,
) -> ComponentResult<impl IntoResponse> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if let Some(component) = Component::get_by_id(&ctx, &request.component_id).await? {
        if let Some(schema) = component.schema(&ctx).await? {
            if let Some(default_variant_id) = schema.default_schema_variant_id() {
                migrate_component_to_schema_variant(
                    &ctx,
                    request.component_id,
                    *default_variant_id,
                )
                .await?;

                WsEvent::component_updated(&ctx, request.component_id)
                    .await?
                    .publish_on_commit(&ctx)
                    .await?;

                ctx.commit().await?;
            }
        }
    }

    let response = axum::response::Response::builder();
    Ok(response.body(axum::body::Empty::new())?)
}
