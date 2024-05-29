use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};

use dal::component::ComponentGeometry;
use dal::diagram::SummaryDiagramComponent;
use dal::{ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetTypeRequest {
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub overridden_geometry: Option<ComponentGeometry>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn set_type(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(SetTypeRequest {
        component_id,
        component_type,
        overridden_geometry,
        visibility,
    }): Json<SetTypeRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut component = Component::get_by_id(&ctx, component_id).await?;

    component.set_type(&ctx, component_type).await?;
    if let Some(geometry) = overridden_geometry {
        component
            .set_geometry(
                &ctx,
                geometry.x,
                geometry.y,
                geometry.width,
                geometry.height,
            )
            .await?;
    }

    let component = Component::get_by_id(&ctx, component_id).await?;
    let payload: SummaryDiagramComponent =
        SummaryDiagramComponent::assemble(&ctx, &component).await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let component_schema = component.schema(&ctx).await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "set_component_type",
        serde_json::json!({
            "how": "/component/set_type",
            "component_id": component.id(),
            "component_schema_name": component_schema.name(),
            "new_component_type": component_type,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
