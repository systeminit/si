use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::component::{ComponentError, ComponentResult};
use axum::extract::OriginalUri;
use axum::response::IntoResponse;
use axum::Json;
use dal::{ChangeSet, Component, ComponentId, SchemaVariant, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn upgrade(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<UpgradeComponentRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let current_component = Component::get_by_id(&ctx, request.component_id).await?;
    let current_schema_variant = current_component.schema_variant(&ctx).await?;
    let schema = current_schema_variant.schema(&ctx).await?;
    let default_schema_variant = SchemaVariant::get_default_for_schema(&ctx, schema.id()).await?;

    // This is just a check to see if someone has made a request incorrectly!
    if current_schema_variant.id() == default_schema_variant.id() {
        return Err(ComponentError::SchemaVariantUpgradeSkipped);
    }

    current_component
        .upgrade_to_new_variant(&ctx, default_schema_variant.id())
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "upgrade_component",
        serde_json::json!({
            "how": "/component/upgrade_component",
            "component_id": request.component_id,
            "component_schema_variant_id": current_schema_variant.id(),
            "new_schema_variant_id": default_schema_variant.id(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    WsEvent::component_upgraded(&ctx, request.component_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
