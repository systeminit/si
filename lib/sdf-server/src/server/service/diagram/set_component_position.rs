use axum::Json;
use dal::{ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElementPositions {
    pub component_id: ComponentId,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub positions: Vec<ElementPositions>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionResponse {
    pub component: Component,
}

pub async fn set_component_position(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetComponentPositionRequest>,
) -> DiagramResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut components: Vec<Component> = vec![];
    for element in request.positions {
        let mut component = Component::get_by_id(&ctx, element.component_id).await?;

        let (width, height) = {
            let mut size = (None, None);

            let component_type = component.get_type(&ctx).await?;

            if component_type != ComponentType::Component {
                size = (
                    element
                        .width
                        .or_else(|| component.width().map(|v| v.to_string())),
                    element
                        .height
                        .or_else(|| component.height().map(|v| v.to_string())),
                );
            }

            size
        };

        component
            .set_geometry(&ctx, element.x, element.y, width, height)
            .await?;
        components.push(component);
    }
    let user_id = ChangeSet::extract_userid_from_context(&ctx).await;

    WsEvent::set_component_position(&ctx, ctx.change_set_id(), &components, user_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
