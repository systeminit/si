use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::component::frame::Frame;
use dal::component::{DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::{
    generate_name, ChangeSet, Component, ComponentId, SchemaId, SchemaVariant, Visibility, WsEvent,
};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::DiagramResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentRequest {
    pub schema_id: SchemaId,
    pub parent_id: Option<ComponentId>,
    pub x: String,
    pub y: String,
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
    Json(request): Json<CreateComponentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let name = generate_name();

    let variant = SchemaVariant::get_default_for_schema(&ctx, request.schema_id).await?;

    let component = Component::new(&ctx, &name, variant.id()).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_component",
        serde_json::json!({
            "how": "/diagram/create_component",
            "component_id": component.id(),
            "component_name": name.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    let component = component
        .set_geometry(
            &ctx,
            request.x.clone(),
            request.y.clone(),
            Some(DEFAULT_COMPONENT_WIDTH),
            Some(DEFAULT_COMPONENT_HEIGHT),
        )
        .await?;

    if let Some(frame_id) = request.parent_id {
        Frame::attach_child_to_parent(&ctx, frame_id, component.id()).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "component_attached_to_frame",
            serde_json::json!({
                "how": "/diagram/create_component",
                "component_id": component.id(),
                "parent_id": frame_id.clone(),
                "change_set_id": ctx.change_set_id(),
            }),
        );
    }

    WsEvent::component_created(&ctx, component.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "component_created",
        serde_json::json!({
            "how": "/diagram/create_component",
            "component_id": component.id(),
            "component_name": name.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

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
