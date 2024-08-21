use std::collections::HashMap;

use axum::extract::{Host, OriginalUri};
use axum::{response::IntoResponse, Json};
use dal::change_status::ChangeStatus;
use serde::{Deserialize, Serialize};

use dal::component::frame::Frame;
use dal::component::{DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::{
    generate_name, ChangeSet, Component, ComponentId, SchemaVariant, SchemaVariantId, Visibility,
    WsEvent,
};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::DiagramResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentRequest {
    pub schema_variant_id: SchemaVariantId,
    pub parent_id: Option<ComponentId>,
    pub x: String,
    pub y: String,
    pub height: Option<String>,
    pub width: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentResponse {
    pub component_id: ComponentId,
}

pub async fn create_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateComponentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let name = generate_name();

    let variant = SchemaVariant::get_by_id_or_error(&ctx, request.schema_variant_id).await?;

    let mut component = Component::new(&ctx, &name, variant.id()).await?;

    component
        .set_geometry(
            &ctx,
            request.x.clone(),
            request.y.clone(),
            request
                .width
                .or_else(|| Some(DEFAULT_COMPONENT_WIDTH.to_string())),
            request
                .height
                .or_else(|| Some(DEFAULT_COMPONENT_HEIGHT.to_string())),
        )
        .await?;

    if let Some(frame_id) = request.parent_id {
        Frame::upsert_parent(&ctx, component.id(), frame_id).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "component_attached_to_frame",
            serde_json::json!({
                "how": "/diagram/create_component",
                "component_id": component.id(),
                "parent_id": frame_id.clone(),
                "change_set_id": ctx.change_set_id(),
            }),
        );
    } else {
        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "component_created",
            serde_json::json!({
                "how": "/diagram/create_component",
                "component_id": component.id(),
                "component_name": name.clone(),
                "change_set_id": ctx.change_set_id(),
            }),
        );
    }

    let mut diagram_sockets = HashMap::new();
    let payload = component
        .into_frontend_type(&ctx, ChangeStatus::Added, &mut diagram_sockets)
        .await?;
    WsEvent::component_created(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    response = response.header("content-type", "application/json");
    Ok(
        response.body(serde_json::to_string(&CreateComponentResponse {
            component_id: component.id(),
        })?)?,
    )
}
