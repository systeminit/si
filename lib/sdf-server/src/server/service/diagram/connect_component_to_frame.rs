use std::collections::{HashMap, VecDeque};

use async_recursion::async_recursion;
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use hyper::http::Uri;
use serde::{Deserialize, Serialize};

use dal::edge::{EdgeKind, EdgeObjectId, VertexObjectKind};
use dal::job::definition::DependentValuesUpdate;
use dal::socket::{SocketEdgeKind, SocketKind};
use dal::{
    node::NodeId, AttributeReadContext, AttributeValue, ChangeSet, Component, ComponentError,
    Connection, DalContext, Edge, EdgeError, ExternalProvider, InternalProvider, SocketId,
    StandardModel, Visibility, WsEvent,
};
use dal::{ComponentType, Socket};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

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

// Create all valid connections between parent and child sockets
pub async fn connect_component_sockets_to_frame(
    ctx: &DalContext,
    parent_node_id: NodeId,
    child_node_id: NodeId,
    original_uri: &Uri,
    posthog_client: &crate::server::state::PosthogClient,
) -> DiagramResult<()> {
    // We stored connected sockets to ensure we connect children's sockets only to the nearest valid ancestor socket
    let mut connected_sockets_for_node_id: HashMap<NodeId, Vec<SocketId>> = HashMap::new();

    connected_sockets_for_node_id
        .entry(child_node_id)
        .or_default();
    connected_sockets_for_node_id
        .entry(parent_node_id)
        .or_default();

    Connection::new_to_parent(ctx, child_node_id, parent_node_id).await?;

    connect_component_sockets_to_frame_inner(
        ctx,
        parent_node_id,
        child_node_id,
        original_uri,
        posthog_client,
        &mut connected_sockets_for_node_id,
    )
    .await?;

    // Now we have to propagate the children of child_node_id
    let mut sorted_children: VecDeque<(NodeId, NodeId)> =
        Edge::list_children_for_node(ctx, child_node_id)
            .await?
            .into_iter()
            .map(|grandchild_node_id| (child_node_id, grandchild_node_id))
            .collect();

    // Goes down new child's children list, trying to connect them to their new ancestors' Configuration Sockets
    while let Some((parent_node_id, child_node_id)) = sorted_children.pop_front() {
        connect_component_sockets_to_frame_inner(
            ctx,
            parent_node_id,
            child_node_id,
            original_uri,
            posthog_client,
            &mut HashMap::new(),
        )
        .await?;

        sorted_children.extend(
            Edge::list_children_for_node(ctx, child_node_id)
                .await?
                .into_iter()
                .map(|grandchild_node_id| (child_node_id, grandchild_node_id)),
        );
    }

    Ok(())
}

#[async_recursion]
async fn connect_component_sockets_to_frame_inner(
    ctx: &DalContext,
    parent_node_id: NodeId,
    child_node_id: NodeId,
    original_uri: &Uri,
    posthog_client: &crate::server::state::PosthogClient,
    connected_sockets_for_node_id: &mut HashMap<NodeId, Vec<SocketId>>,
) -> DiagramResult<()> {
    let parent_component = Component::find_for_node(ctx, parent_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(parent_node_id))?;

    let parent_sockets = Socket::list_for_component(ctx, *parent_component.id()).await?;

    let child_component = Component::find_for_node(ctx, child_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(child_node_id))?;

    for parent_socket in &parent_sockets {
        if parent_socket.kind() == &SocketKind::Frame {
            continue;
        }

        match parent_component.get_type(ctx).await? {
            component_type @ ComponentType::Component => {
                return Err(DiagramError::InvalidComponentTypeForFrame(component_type))
            }
            ComponentType::AggregationFrame => {
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
                            ctx.access_builder(),
                            *ctx.visibility(),
                            vec![*attribute_value.id()],
                        ))
                        .await?;
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
                            ctx.access_builder(),
                            *ctx.visibility(),
                            vec![*attribute_value.id()],
                        ))
                        .await?;
                    }
                }
            }
            component_type @ (ComponentType::ConfigurationFrameDown
            | ComponentType::ConfigurationFrameUp) => {
                let child_sockets = Socket::list_for_component(ctx, *child_component.id()).await?;

                for child_socket in &child_sockets {
                    // Skip child sockets corresponding to frames.
                    if child_socket.kind() == &SocketKind::Frame {
                        continue;
                    }

                    // Configuration frames down and up behave similarly, only connecting either child to parent vs
                    // parent to child sockets. So we assign destination and source entities here
                    let (
                        source_node_id,
                        source_socket,
                        dest_node_id,
                        destination_component_id,
                        dest_socket,
                    ) = if component_type == ComponentType::ConfigurationFrameDown {
                        let used_socket = connected_sockets_for_node_id
                            .get(&child_node_id)
                            .into_iter()
                            .flatten()
                            .any(|socket_id| child_socket.id() == socket_id);
                        if used_socket {
                            continue;
                        }

                        (
                            parent_node_id,
                            parent_socket,
                            child_node_id,
                            *child_component.id(),
                            child_socket,
                        )
                    } else {
                        (
                            child_node_id,
                            child_socket,
                            parent_node_id,
                            *parent_component.id(),
                            parent_socket,
                        )
                    };

                    if let (Some(source_provider), Some(dest_provider)) = (
                        source_socket.external_provider(ctx).await?,
                        dest_socket.internal_provider(ctx).await?,
                    ) {
                        // TODO(victor): Refactor to match on connection annotations.
                        if source_provider.name() == dest_provider.name() {
                            connected_sockets_for_node_id
                                .entry(dest_node_id)
                                .or_default()
                                .push(*dest_socket.id());

                            Connection::new(
                                ctx,
                                source_node_id,
                                *source_socket.id(),
                                dest_node_id,
                                *dest_socket.id(),
                                EdgeKind::Configuration,
                            )
                            .await?;

                            let dest_socket_internal_provider =
                                InternalProvider::find_explicit_for_socket(ctx, *dest_socket.id())
                                    .await?
                                    .ok_or(DiagramError::InternalProviderNotFoundForSocket(
                                        *dest_socket.id(),
                                    ))?;

                            let dest_attribute_value_context = AttributeReadContext {
                                internal_provider_id: Some(*dest_socket_internal_provider.id()),
                                component_id: Some(destination_component_id),
                                ..Default::default()
                            };
                            let mut dest_attribute_value =
                                AttributeValue::find_for_context(ctx, dest_attribute_value_context)
                                    .await?
                                    .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                        dest_attribute_value_context,
                                    ))?;

                            dest_attribute_value
                                .update_from_prototype_function(ctx)
                                .await?;

                            ctx.enqueue_job(DependentValuesUpdate::new(
                                ctx.access_builder(),
                                *ctx.visibility(),
                                vec![*dest_attribute_value.id()],
                            ))
                            .await?;
                        }
                    }
                }
            }
        };
    }

    if let Some(grandparent_id) =
        Edge::get_parent_for_component(ctx, *parent_component.id()).await?
    {
        let grandparent = Component::get_by_id(ctx, &grandparent_id)
            .await?
            .ok_or(ComponentError::NotFound(grandparent_id))?;
        let ty = grandparent.get_type(ctx).await?;

        let grandparent = grandparent
            .node(ctx)
            .await?
            .pop()
            .ok_or(ComponentError::NodeNotFoundForComponent(grandparent_id))?;
        match ty {
            ComponentType::Component => {}
            ComponentType::ConfigurationFrameDown | ComponentType::ConfigurationFrameUp => {
                connect_component_sockets_to_frame_inner(
                    ctx,
                    *grandparent.id(),
                    child_node_id,
                    original_uri,
                    posthog_client,
                    connected_sockets_for_node_id,
                )
                .await?
            }
            ComponentType::AggregationFrame => unimplemented!(),
        }
    }

    {
        let child_schema = child_component
            .schema(ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        let parent_schema = parent_component
            .schema(ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        let from_socket = Socket::find_frame_socket_for_node(
            ctx,
            child_node_id,
            SocketEdgeKind::ConfigurationOutput,
        )
        .await?;

        let to_socket = Socket::find_frame_socket_for_node(
            ctx,
            parent_node_id,
            SocketEdgeKind::ConfigurationInput,
        )
        .await?;

        track(
            posthog_client,
            ctx,
            original_uri,
            "component_connected_to_frame",
            serde_json::json!({
                        "parent_component_id": parent_component.id(),
                        "parent_component_schema_name": parent_schema.name(),
                        "parent_socket_id": to_socket.id(),
                        "parent_socket_name": to_socket.name(),
                        "child_component_id": child_component.id(),
                        "child_component_schema_name": child_schema.name(),
                        "child_socket_id": from_socket.id(),
                        "child_socket_name": from_socket.name(),
            }),
        );
    }
    Ok(())
}

/// Create a [`Connection`](dal::Connection) with a _to_ [`Socket`](dal::Socket) and
/// [`Node`](dal::Node) and a _from_ [`Socket`](dal::Socket) and [`Node`](dal::Node).
/// Creating a change set if on head.
pub async fn connect_component_to_frame(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateFrameConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    // Connect children to parent through frame edge
    connect_component_sockets_to_frame(
        &ctx,
        request.parent_node_id,
        request.child_node_id,
        &original_uri,
        &posthog_client,
    )
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response
        .header("content-type", "application/json")
        .body("{}".to_owned())?)
}
