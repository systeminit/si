use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::edge::{EdgeKind, EdgeObjectId, VertexObjectKind};
use dal::job::definition::DependentValuesUpdate;
use dal::socket::{SocketEdgeKind, SocketKind};
use dal::{
    node::NodeId, AttributeReadContext, AttributeValue, ChangeSet, Component, Connection,
    DalContext, Edge, EdgeError, ExternalProvider, InternalProvider, InternalProviderId, Node,
    PropId, StandardModel, Visibility, WsEvent,
};
use dal::{ComponentType, Socket};
use serde::{Deserialize, Serialize};

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
// TODO(victor,paul) We should tidy up this function after the feature stabilizes a bit
pub async fn connect_component_sockets_to_frame(
    ctx: &DalContext,
    parent_node_id: NodeId,
    child_node_id: NodeId,
) -> DiagramResult<impl IntoResponse> {
    let parent_component = Component::find_for_node(ctx, parent_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(parent_node_id))?;
    let parent_sockets = Socket::list_for_component(ctx, *parent_component.id()).await?;

    let child_component = Component::find_for_node(ctx, child_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(child_node_id))?;
    let child_sockets = Socket::list_for_component(ctx, *child_component.id()).await?;

    let aggregation_frame = match parent_component.get_type(ctx).await? {
        ComponentType::AggregationFrame => true,
        ComponentType::ConfigurationFrame => false,
        component_type => return Err(DiagramError::InvalidComponentTypeForFrame(component_type)),
    };

    for parent_socket in parent_sockets {
        if parent_socket.kind() == &SocketKind::Frame {
            continue;
        }

        if aggregation_frame {
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
        } else if let Some(parent_provider) = parent_socket.external_provider(ctx).await? {
            for child_socket in &child_sockets {
                // Skip child sockets corresponding to frames.
                if child_socket.kind() == &SocketKind::Frame {
                    continue;
                }

                if let Some(child_provider) = child_socket.internal_provider(ctx).await? {
                    // TODO(nick): once type definitions used for providers, we should not
                    // match on name.
                    if parent_provider.name() == child_provider.name() {
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
                            ctx.access_builder(),
                            *ctx.visibility(),
                            vec![*attribute_value.id()],
                        ))
                        .await?;
                    }
                }
            }
        }
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

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(&ctx).await?, None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    // Connect children to parent through frame edge
    let from_socket = Socket::find_frame_socket_for_node(
        &ctx,
        request.child_node_id,
        SocketEdgeKind::ConfigurationOutput,
    )
    .await?;
    let to_socket = Socket::find_frame_socket_for_node(
        &ctx,
        request.parent_node_id,
        SocketEdgeKind::ConfigurationInput,
    )
    .await?;

    let connection = Connection::new(
        &ctx,
        request.child_node_id,
        *from_socket.id(),
        request.parent_node_id,
        *to_socket.id(),
        EdgeKind::Symbolic,
    )
    .await?;

    connect_component_sockets_to_frame(&ctx, request.parent_node_id, request.child_node_id).await?;

    let child_comp = Node::get_by_id(&ctx, &request.child_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(request.child_node_id))?
        .component(&ctx)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let child_schema = child_comp
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let parent_comp = Node::get_by_id(&ctx, &request.parent_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(request.parent_node_id))?
        .component(&ctx)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let parent_schema = parent_comp
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "component_connected_to_frame",
        serde_json::json!({
                    "parent_component_id": parent_comp.id(),
                    "parent_component_schema_name": parent_schema.name(),
                    "parent_socket_id": to_socket.id(),
                    "parent_socket_name": to_socket.name(),
                    "child_component_id": child_comp.id(),
                    "child_component_schema_name": child_schema.name(),
                    "child_socket_id": from_socket.id(),
                    "child_socket_name": from_socket.name(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(
        response.body(serde_json::to_string(&CreateFrameConnectionResponse {
            connection,
        })?)?,
    )
}
