use axum::Json;
use dal::edge::{EdgeKind, EdgeObjectId, VertexObjectKind};
use dal::job::definition::DependentValuesUpdate;
use dal::socket::SocketEdgeKind;
use dal::{
    node::NodeId, AttributeReadContext, AttributeValue, Component, Connection, DalContext, Edge,
    EdgeError, ExternalProvider, InternalProvider, InternalProviderId, Node, PropId, StandardModel,
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
        EdgeKind::Symbolic,
    )
    .await?;

    connect_component_sockets_to_frame(&ctx, request.parent_node_id, request.child_node_id).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(CreateFrameConnectionResponse { connection }))
}

// Create all valid connections between parent and child sockets
// TODO(victor,paul) We should tidy up this function after the feature stabilizes a bit
pub async fn connect_component_sockets_to_frame(
    ctx: &DalContext,
    parent_node_id: NodeId,
    child_node_id: NodeId,
) -> DiagramResult<()> {
    let parent_component = Component::find_for_node(ctx, parent_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(parent_node_id))?;

    let parent_schema_variant = parent_component
        .schema_variant(ctx)
        .await?
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let parent_sockets = parent_schema_variant.sockets(ctx).await?;

    let child_component = Component::find_for_node(ctx, child_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(child_node_id))?;

    let child_sockets = child_component
        .schema_variant(ctx)
        .await?
        .ok_or(DiagramError::SchemaVariantNotFound)?
        .sockets(ctx)
        .await?;

    for parent_socket in parent_sockets {
        if parent_socket.name() == "Frame" {
            continue;
        }

        let frame_type_opt = parent_component
            .find_value_by_json_pointer::<String>(ctx, "/root/si/type")
            .await?;

        // TODO(victor) This could be improved, frame_type should be compared to an enum
        if let Some(frame_type) = frame_type_opt {
            if frame_type == "aggregationFrame" {
                match *parent_socket.edge_kind() {
                    SocketEdgeKind::ConfigurationInput => {
                        let provider =
                            InternalProvider::find_explicit_for_socket(ctx, *parent_socket.id())
                                .await?
                                .ok_or(EdgeError::InternalProviderNotFoundForSocket(
                                    *parent_socket.id(),
                                ))?;

                        // We don't want to connect the provider when we are not using configuration edge kind
                        Edge::connect_internal_providers_for_components(
                            ctx,
                            *provider.id(),
                            *child_component.id(),
                            *parent_component.id(),
                        )
                        .await?;

                        Edge::new(
                            ctx,
                            EdgeKind::Configuration,
                            child_node_id,
                            VertexObjectKind::Configuration,
                            EdgeObjectId::from(*child_component.id()),
                            *parent_socket.id(),
                            parent_node_id,
                            VertexObjectKind::Configuration,
                            EdgeObjectId::from(*parent_component.id()),
                            *parent_socket.id(),
                        )
                        .await?;

                        let attribute_value_context = AttributeReadContext {
                            component_id: Some(*parent_component.id()),
                            internal_provider_id: Some(*provider.id()),
                            ..Default::default()
                        };

                        let attribute_value =
                            AttributeValue::find_for_context(ctx, attribute_value_context)
                                .await?
                                .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                    attribute_value_context,
                                ))?;

                        ctx.enqueue_job(DependentValuesUpdate::new(
                            ctx,
                            vec![*attribute_value.id()],
                        ))
                        .await;
                    }
                    SocketEdgeKind::ConfigurationOutput => {
                        let provider = ExternalProvider::find_for_socket(ctx, *parent_socket.id())
                            .await?
                            .ok_or(EdgeError::ExternalProviderNotFoundForSocket(
                                *parent_socket.id(),
                            ))?;

                        Edge::connect_external_providers_for_components(
                            ctx,
                            *provider.id(),
                            *parent_component.id(),
                            *child_component.id(),
                        )
                        .await?;

                        Edge::new(
                            ctx,
                            EdgeKind::Configuration,
                            parent_node_id,
                            VertexObjectKind::Configuration,
                            EdgeObjectId::from(*parent_component.id()),
                            *parent_socket.id(),
                            child_node_id,
                            VertexObjectKind::Configuration,
                            EdgeObjectId::from(*child_component.id()),
                            *parent_socket.id(),
                        )
                        .await?;

                        let attribute_value_context = AttributeReadContext {
                            component_id: Some(*child_component.id()),
                            external_provider_id: Some(*provider.id()),
                            ..Default::default()
                        };

                        let attribute_value =
                            AttributeValue::find_for_context(ctx, attribute_value_context)
                                .await?
                                .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                    attribute_value_context,
                                ))?;

                        ctx.enqueue_job(DependentValuesUpdate::new(
                            ctx,
                            vec![*attribute_value.id()],
                        ))
                        .await;
                    }
                }
            } else if let Some(parent_provider) = parent_socket.external_provider(ctx).await? {
                for child_socket in &child_sockets {
                    if let Some(child_provider) = child_socket.internal_provider(ctx).await? {
                        if parent_provider.name() != "Frame"
                            && parent_provider.name() == child_provider.name()
                        {
                            Connection::new(
                                ctx,
                                parent_node_id,
                                *parent_socket.id(),
                                child_node_id,
                                *child_socket.id(),
                                EdgeKind::Configuration,
                            )
                            .await?;

                            let attribute_read_context = AttributeReadContext {
                                prop_id: Some(PropId::NONE),
                                internal_provider_id: Some(InternalProviderId::NONE),
                                external_provider_id: Some(*parent_provider.id()),
                                component_id: Some(*parent_component.id()),
                            };

                            let attribute_value =
                                AttributeValue::find_for_context(ctx, attribute_read_context)
                                    .await?
                                    .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                        attribute_read_context,
                                    ))?;

                            ctx.enqueue_job(DependentValuesUpdate::new(
                                ctx,
                                vec![*attribute_value.id()],
                            ))
                            .await;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
