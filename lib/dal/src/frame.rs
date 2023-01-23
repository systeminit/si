//! This module contains functionality related to the "frame" concept.

use thiserror::Error;

use crate::edge::{EdgeKind, EdgeObjectId, VertexObjectKind};
use crate::job::definition::DependentValuesUpdate;
use crate::socket::{SocketEdgeKind, SocketError, SocketKind};
use crate::{
    node::NodeId, AttributeReadContext, AttributeValue, AttributeValueError, Component,
    ComponentError, ComponentId, ComponentType, Connection, DalContext, DiagramError, Edge,
    EdgeError, ExternalProvider, ExternalProviderError, InternalProvider, InternalProviderError,
    InternalProviderId, PropId, Socket, StandardModel,
};

#[derive(Error, Debug)]
pub enum FrameError {
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    Diagram(#[from] DiagramError),
    #[error(transparent)]
    Edge(#[from] EdgeError),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error(transparent)]
    Socket(#[from] SocketError),

    /// No [`AttributeValue`](crate::AttributeValue) was found for the given
    /// [`AttributeReadContext`](crate::AttributeReadContext).
    #[error("attribute value not found for attribute read context: {0:?}")]
    AttributeValueNotFoundForReadContext(AttributeReadContext),
    /// No [`Component`](crate::Component) was found for a given [`NodeId`](crate::Node).
    #[error("no component found for node id: {0}")]
    ComponentNotFoundForNodeId(NodeId),
    /// The [`ComponentType`](crate::ComponentType) is invalid for the [`frame`](crate::Frame).
    #[error("frame cannot be of component type: {0:?}")]
    InvalidComponentTypeForFrame(ComponentType),
    /// The [`Node`](crate::Node) was not found for the provided [`NodeId`](crate::Node).
    #[error("node not found by id: {0}")]
    NodeNotFound(NodeId),
}

pub type FrameResult<T> = Result<T, FrameError>;

/// Frame functionality is behavioral-based and does not correlate to a specific struct or table
/// definition. Thus, this unit struct provides co-located functionality related to the "frame"
/// concept.
pub struct Frame;

impl Frame {
    /// Connect a [`frame`](crate::Frame) [`Node`](crate::Node) and child [`Node`](crate::Node).
    /// Return the [`symbolic`](crate::EdgeKind::Symbolic) [`Connection`](crate::Connection) once
    /// all underlying connections are created.
    pub async fn connect(
        ctx: &DalContext,
        frame_node_id: NodeId,
        child_node_id: NodeId,
    ) -> FrameResult<Connection> {
        let frame_component = Component::find_for_node(ctx, frame_node_id)
            .await?
            .ok_or(FrameError::ComponentNotFoundForNodeId(child_node_id))?;
        let frame_component_id = *frame_component.id();
        let child_component_id = Component::find_for_node(ctx, child_node_id)
            .await?
            .map(|c| *c.id())
            .ok_or(FrameError::ComponentNotFoundForNodeId(child_node_id))?;

        // Find the appropriate sockets for the symbolic connection.
        let output_socket_on_child = Socket::find_frame_socket_for_component(
            ctx,
            child_component_id,
            SocketEdgeKind::ConfigurationOutput,
        )
        .await?;
        let input_socket_on_frame = Socket::find_frame_socket_for_component(
            ctx,
            frame_component_id,
            SocketEdgeKind::ConfigurationInput,
        )
        .await?;

        // Create the symbolic connection first.
        let symbolic_connection = Connection::new(
            &ctx,
            child_node_id,
            *output_socket_on_child.id(),
            frame_node_id,
            *input_socket_on_frame.id(),
            EdgeKind::Symbolic,
        )
        .await?;

        let component_type = frame_component.get_type(ctx).await?;
        match component_type {
            ComponentType::AggregationFrame => {
                Self::perform_connection_for_aggregation_frame(
                    ctx,
                    frame_node_id,
                    *frame_component.id(),
                    child_node_id,
                    child_component_id,
                )
                .await?
            }
            ComponentType::ConfigurationFrame => {
                Self::perform_connection_for_configuration_frame(
                    ctx,
                    frame_node_id,
                    *frame_component.id(),
                    child_node_id,
                    child_component_id,
                )
                .await?
            }
            ComponentType::Component => {
                return Err(FrameError::InvalidComponentTypeForFrame(component_type));
            }
        }

        Ok(symbolic_connection)
    }

    /// This _private_ method performs the connection between a
    /// [`aggregation`](crate::ComponentType::AggregationFrame) [`frame`](crate::Frame) and a
    /// child [`Component`](crate::Component).
    async fn perform_connection_for_aggregation_frame(
        ctx: &DalContext,
        frame_node_id: NodeId,
        frame_component_id: ComponentId,
        child_node_id: NodeId,
        child_component_id: ComponentId,
    ) -> FrameResult<()> {
        let frame_sockets = Socket::list_for_component(ctx, frame_component_id).await?;

        for frame_socket in frame_sockets {
            // Ensure that we only work with sockets that aren't the special frame sockets.
            if frame_socket.kind() == &SocketKind::Frame {
                continue;
            }

            match *frame_socket.edge_kind() {
                SocketEdgeKind::ConfigurationInput => {
                    let provider =
                        InternalProvider::find_explicit_for_socket(ctx, *frame_socket.id())
                            .await?
                            .ok_or(EdgeError::InternalProviderNotFoundForSocket(
                                *frame_socket.id(),
                            ))?;

                    // We don't want to connect the provider when we are not using configuration edge kind
                    Edge::connect_internal_providers_for_components(
                        ctx,
                        *provider.id(),
                        child_component_id,
                        frame_component_id,
                    )
                    .await?;

                    Edge::new(
                        ctx,
                        EdgeKind::Configuration,
                        child_node_id,
                        VertexObjectKind::Configuration,
                        EdgeObjectId::from(child_component_id),
                        *frame_socket.id(),
                        frame_node_id,
                        VertexObjectKind::Configuration,
                        EdgeObjectId::from(frame_component_id),
                        *frame_socket.id(),
                    )
                    .await?;

                    let attribute_value_context = AttributeReadContext {
                        component_id: Some(frame_component_id),
                        internal_provider_id: Some(*provider.id()),
                        ..Default::default()
                    };

                    let attribute_value =
                        AttributeValue::find_for_context(ctx, attribute_value_context)
                            .await?
                            .ok_or(FrameError::AttributeValueNotFoundForReadContext(
                                attribute_value_context,
                            ))?;

                    ctx.enqueue_job(DependentValuesUpdate::new(ctx, vec![*attribute_value.id()]))
                        .await;
                }
                SocketEdgeKind::ConfigurationOutput => {
                    let provider = ExternalProvider::find_for_socket(ctx, *frame_socket.id())
                        .await?
                        .ok_or(EdgeError::ExternalProviderNotFoundForSocket(
                            *frame_socket.id(),
                        ))?;

                    Edge::connect_external_providers_for_components(
                        ctx,
                        *provider.id(),
                        frame_component_id,
                        child_component_id,
                    )
                    .await?;

                    Edge::new(
                        ctx,
                        EdgeKind::Configuration,
                        frame_node_id,
                        VertexObjectKind::Configuration,
                        EdgeObjectId::from(frame_component_id),
                        *frame_socket.id(),
                        child_node_id,
                        VertexObjectKind::Configuration,
                        EdgeObjectId::from(child_component_id),
                        *frame_socket.id(),
                    )
                    .await?;

                    let attribute_value_context = AttributeReadContext {
                        component_id: Some(child_component_id),
                        external_provider_id: Some(*provider.id()),
                        ..Default::default()
                    };

                    let attribute_value =
                        AttributeValue::find_for_context(ctx, attribute_value_context)
                            .await?
                            .ok_or(FrameError::AttributeValueNotFoundForReadContext(
                                attribute_value_context,
                            ))?;

                    ctx.enqueue_job(DependentValuesUpdate::new(ctx, vec![*attribute_value.id()]))
                        .await;
                }
            }
        }
        Ok(())
    }

    /// This _private_ method performs the connection between a
    /// [`configuration`](crate::ComponentType::ConfigurationFrame) [`frame`](crate::Frame) and a
    /// child [`Component`](crate::Component).
    async fn perform_connection_for_configuration_frame(
        ctx: &DalContext,
        frame_node_id: NodeId,
        frame_component_id: ComponentId,
        child_node_id: NodeId,
        child_component_id: ComponentId,
    ) -> FrameResult<()> {
        let frame_sockets = Socket::list_for_component(ctx, frame_component_id).await?;
        let child_sockets = Socket::list_for_component(ctx, child_component_id).await?;

        for frame_socket in frame_sockets {
            // Ensure that we only work with sockets that aren't the special frame sockets.
            if frame_socket.kind() == &SocketKind::Frame {
                continue;
            }

            if let Some(parent_provider) = frame_socket.external_provider(ctx).await? {
                for child_socket in &child_sockets {
                    if let Some(child_provider) = child_socket.internal_provider(ctx).await? {
                        if parent_provider.name() != "Frame"
                            && parent_provider.name() == child_provider.name()
                        {
                            Connection::new(
                                ctx,
                                frame_node_id,
                                *frame_socket.id(),
                                child_node_id,
                                *child_socket.id(),
                                EdgeKind::Configuration,
                            )
                            .await?;

                            let attribute_read_context = AttributeReadContext {
                                prop_id: Some(PropId::NONE),
                                internal_provider_id: Some(InternalProviderId::NONE),
                                external_provider_id: Some(*parent_provider.id()),
                                component_id: Some(frame_component_id),
                            };

                            let attribute_value =
                                AttributeValue::find_for_context(ctx, attribute_read_context)
                                    .await?
                                    .ok_or(FrameError::AttributeValueNotFoundForReadContext(
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
        Ok(())
    }
}
