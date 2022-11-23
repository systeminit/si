use axum::Json;
use dal::{node::NodeId, Connection, Node, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{DiagramError, DiagramResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionResponse {
    pub connection: Connection,
}

/// Create a [`Connection`](dal::Connection) with a _to_ [`Socket`](dal::Socket) and
/// [`Node`](dal::Node) and a _from_ [`Socket`](dal::Socket) and [`Node`](dal::Node).
pub async fn connect_component_to_frame(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateConnectionRequest>,
) -> DiagramResult<Json<CreateConnectionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let from_socket_id = {
        let from_node = Node::get_by_id(&ctx, &request.from_node_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(request.from_node_id))?;

        let from_component = from_node
            .component(&ctx)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let schema_variant = from_component
            .schema_variant(&ctx)
            .await?
            .ok_or(DiagramError::SchemaVariantNotFound)?;

        let from_sockets = schema_variant.sockets(&ctx).await?;

        let mut from_socket_id = None;

        for socket in from_sockets {
            if let Some(provider) = socket.external_provider(&ctx).await? {
                if provider.name() == "Frame" {
                    from_socket_id = Some(*socket.id());
                    break;
                }
            }
        }

        match from_socket_id {
            None => {
                return Err(DiagramError::FrameSocketNotFound(*schema_variant.id()));
            }
            Some(socket_id) => socket_id,
        }
    };

    let to_socket_id = {
        let node = Node::get_by_id(&ctx, &request.to_node_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(request.to_node_id))?;

        let component = node
            .component(&ctx)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let schema_variant = component
            .schema_variant(&ctx)
            .await?
            .ok_or(DiagramError::SchemaVariantNotFound)?;

        let sockets = schema_variant.sockets(&ctx).await?;

        let mut socket_id = None;

        for socket in sockets {
            if let Some(provider) = socket.internal_provider(&ctx).await? {
                if provider.name() == "Frame" {
                    socket_id = Some(*socket.id());
                    break;
                }
            }
        }

        match socket_id {
            None => {
                return Err(DiagramError::FrameSocketNotFound(*schema_variant.id()));
            }
            Some(socket_id) => socket_id,
        }
    };

    let connection = Connection::new(
        &ctx,
        request.from_node_id,
        from_socket_id,
        request.to_node_id,
        to_socket_id,
    )
    .await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateConnectionResponse { connection }))
}
