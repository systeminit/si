use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::diagram::DiagramError;
use axum::Json;
use dal::socket::SocketEdgeKind;
use dal::{Component, ComponentId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionRequest {
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
pub struct SetNodePositionResponse {
    pub component: Component,
}

pub async fn set_node_position(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetNodePositionRequest>,
) -> DiagramResult<Json<SetNodePositionResponse>> {
    let visibility = Visibility::new_change_set(request.visibility.change_set_pk, true);
    let ctx = builder.build(request_ctx.build(visibility)).await?;

    let mut component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let (width, height) = {
        let sockets = component
            .schema_variant(&ctx)
            .await?
            .ok_or(DiagramError::SchemaVariantNotFound)?
            .sockets(&ctx)
            .await?;

        let mut size = (None, None);

        for s in sockets {
            // If component is a frame, we set the size as either the one from the request or the previous one
            // If we don't do it like this upsert will delete the size on None instead of keeping it as is
            if s.name() == "Frame" && *s.edge_kind() == SocketEdgeKind::ConfigurationInput {
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

    {
        if component.visibility().deleted_at.is_some() {
            component
                .set_geometry(&ctx, &request.x, &request.y, width, height)
                .await?;
        } else {
            let ctx_without_deleted = &ctx.clone_with_new_visibility(Visibility::new_change_set(
                ctx.visibility().change_set_pk,
                false,
            ));

            component
                .set_geometry(ctx_without_deleted, &request.x, &request.y, width, height)
                .await?;
        };
    }

    ctx.commit().await?;

    Ok(Json(SetNodePositionResponse { component }))
}
