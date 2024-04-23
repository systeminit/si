use axum::Json;
use dal::{ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub component_id: ComponentId,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
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
) -> DiagramResult<Json<SetComponentPositionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component = Component::get_by_id(&ctx, request.component_id).await?;
    //let schema_variant_id = Component::schema_variant_id(&ctx, request.component_id).await?;

    let (width, height) = {
        let mut size = (None, None);

        let component_type = component.get_type(&ctx).await?;

        if component_type != ComponentType::Component {
            size = (
                request
                    .width
                    .or_else(|| component.width().map(|v| v.to_string())),
                request
                    .height
                    .or_else(|| component.height().map(|v| v.to_string())),
            );
        }

        size
    };

    // TODO(nick): handle the "deleted" case with the new engine.
    let component = component
        .set_geometry(&ctx, request.x, request.y, width, height)
        .await?;
    // {
    //     if node.visibility().deleted_at.is_some() {
    //         node.set_geometry(&ctx, &request.x, &request.y, width, height)
    //             .await?;
    //     } else {
    //         let ctx_without_deleted = &ctx.clone_with_new_visibility(Visibility::new_change_set(
    //             ctx.visibility().change_set_pk,
    //             false,
    //         ));
    //
    //         node.set_geometry(ctx_without_deleted, &request.x, &request.y, width, height)
    //             .await?;
    //     };
    // }
    let user_id = ChangeSet::extract_userid_from_context(&ctx).await;

    WsEvent::set_component_position(&ctx, ctx.change_set_id(), &component, user_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(SetComponentPositionResponse { component }))
}
