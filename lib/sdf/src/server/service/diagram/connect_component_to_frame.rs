use axum::Json;
use dal::job::definition::DependentValuesUpdate;
use dal::{
    node::NodeId, AttributeReadContext, AttributeValue, Connection, Node, StandardModel,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{DiagramError, DiagramResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFrameConnectionRequest {
    pub child_node_id: NodeId,
    pub parent_node_id: NodeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFrameConnectionResponse {
    pub connection: Connection,
}

/// Create a [`Connection`](dal::Connection) with a _to_ [`Socket`](dal::Socket) and
/// [`Node`](dal::Node) and a _from_ [`Socket`](dal::Socket) and [`Node`](dal::Node).
pub async fn connect_component_to_frame(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateFrameConnectionRequest>,
) -> DiagramResult<Json<CreateFrameConnectionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // Connect children to parent through frame edge
    let from_socket_id = {
        let from_node = Node::get_by_id(&ctx, &request.child_node_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(request.child_node_id))?;

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
        let node = Node::get_by_id(&ctx, &request.parent_node_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(request.parent_node_id))?;

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
        request.child_node_id,
        from_socket_id,
        request.parent_node_id,
        to_socket_id,
    )
    .await?;

    // Create all valid connections between parent output and child inputs
    {
        let parent_node = Node::get_by_id(&ctx, &request.parent_node_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(request.parent_node_id))?;

        let parent_component = parent_node
            .component(&ctx)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let parent_schema_variant = parent_component
            .schema_variant(&ctx)
            .await?
            .ok_or(DiagramError::SchemaVariantNotFound)?;

        let parent_sockets = parent_schema_variant.sockets(&ctx).await?;

        let child_node = Node::get_by_id(&ctx, &request.child_node_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(request.child_node_id))?;

        let child_component = child_node
            .component(&ctx)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let child_schema_variant = child_component
            .schema_variant(&ctx)
            .await?
            .ok_or(DiagramError::SchemaVariantNotFound)?;

        let child_sockets = child_schema_variant.sockets(&ctx).await?;

        for parent_socket in parent_sockets {
            if let Some(parent_provider) = parent_socket.external_provider(&ctx).await? {
                for child_socket in &child_sockets {
                    if let Some(child_provider) = child_socket.internal_provider(&ctx).await? {
                        if parent_provider.name() != "Frame"
                            && parent_provider.name() == child_provider.name()
                        {
                            Connection::new(
                                &ctx,
                                request.parent_node_id,
                                *parent_socket.id(),
                                request.child_node_id,
                                *child_socket.id(),
                            )
                            .await?;

                            let attribute_value_context = AttributeReadContext {
                                component_id: Some(*parent_component.id()),
                                schema_variant_id: Some(*parent_schema_variant.id()),
                                schema_id: Some(
                                    *parent_schema_variant.schema(&ctx).await?.expect("Err").id(),
                                ),
                                external_provider_id: Some(*parent_provider.id()),
                                ..Default::default()
                            };
                            let attribute_value =
                                AttributeValue::find_for_context(&ctx, attribute_value_context)
                                    .await?
                                    .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                        attribute_value_context,
                                    ))?;

                            ctx.enqueue_job(DependentValuesUpdate::new(
                                &ctx,
                                *attribute_value.id(),
                            ))
                            .await;
                        }
                    }
                }
            }
        }
    }

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateFrameConnectionResponse { connection }))
}
