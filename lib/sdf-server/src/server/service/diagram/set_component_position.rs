use axum::Json;
use dal::{Component, ComponentId, InputSocket, Visibility};
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
    let schema_variant_id = Component::schema_variant_id(&ctx, request.component_id).await?;

    let (width, height) = {
        let input_sockets = InputSocket::list(&ctx, schema_variant_id).await?;

        let mut size = (None, None);

        for input_socket in input_sockets {
            // NOTE(nick): the comment below may be out of date, depending on how we handle frames with the new engine.

            // If component is a frame, we set the size as either the one from the request or the previous one
            // If we don't do it like this upsert_by_node_id will delete the size on None instead of keeping it as is
            if input_socket.name() == "Frame" {
                size = (
                    request
                        .width
                        .or_else(|| component.width().map(|v| v.to_string())),
                    request
                        .height
                        .or_else(|| component.height().map(|v| v.to_string())),
                );
                break;
            }
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

    ctx.commit().await?;

    Ok(Json(SetComponentPositionResponse { component }))
}
