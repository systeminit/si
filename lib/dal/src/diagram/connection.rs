use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::diagram::DiagramResult;
use crate::edge::{Edge, EdgeId, EdgeKind};
use crate::socket::{SocketEdgeKind, SocketId};
use crate::{
    node::NodeId, Component, ComponentError, DalContext, DiagramError, Socket, SocketArity,
    StandardModel, User,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Vertex {
    pub node_id: NodeId,
    pub socket_id: SocketId,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: EdgeId,
    pub classification: EdgeKind,
    pub source: Vertex,
    pub destination: Vertex,
    pub created_by: Option<User>,
    pub deleted_by: Option<User>,
}

impl Connection {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        from_node_id: NodeId,
        from_socket_id: SocketId,
        to_node_id: NodeId,
        to_socket_id: SocketId,
        edge_kind: EdgeKind,
    ) -> DiagramResult<Self> {
        let from_component = Component::find_for_node(ctx, from_node_id)
            .await?
            .ok_or(ComponentError::NotFoundForNode(from_node_id))?;
        let from_socket = Socket::get_by_id(ctx, &from_socket_id)
            .await?
            .ok_or(DiagramError::SocketNotFound)?;

        let to_component = Component::find_for_node(ctx, to_node_id)
            .await?
            .ok_or(ComponentError::NotFoundForNode(to_node_id))?;
        let to_socket = Socket::get_by_id(ctx, &to_socket_id)
            .await?
            .ok_or(DiagramError::SocketNotFound)?;

        // Ignores connection if it already exists
        let edges = Edge::list_for_component(ctx, *to_component.id()).await?;
        for edge in &edges {
            let same_sockets =
                edge.tail_socket_id() == from_socket_id && edge.head_socket_id() == to_socket_id;
            let same_nodes =
                edge.tail_node_id() == from_node_id && edge.head_node_id() == to_node_id;
            if same_sockets && same_nodes {
                return Ok(Connection::from_edge(edge));
            }
        }

        info!(
            "Connect: {}({}:{}) -> {}({}:{})",
            from_component.name(ctx).await?,
            from_socket.name(),
            from_socket.arity(),
            to_component.name(ctx).await?,
            to_socket.name(),
            to_socket.arity(),
        );

        if *to_socket.arity() == SocketArity::One {
            // Removes all connections for origin node since we are replacing it

            let replaced_edges = edges
                .iter()
                .filter(|edge| edge.head_socket_id() == to_socket_id);

            for replaced_edge in replaced_edges {
                for edge in &edges {
                    if edge.tail_node_id() == replaced_edge.tail_node_id()
                        || edge.head_node_id() == replaced_edge.tail_node_id()
                    {
                        let socket = Socket::get_by_id(ctx, &edge.head_socket_id())
                            .await?
                            .ok_or(DiagramError::SocketNotFound)?;
                        if socket.name() == "Frame" {
                            edge.clone().delete_by_id(ctx).await?;
                        } else {
                            edge.clone().delete_and_propagate(ctx).await?;
                        }
                    }
                }
            }
        }

        let edge = Edge::new_for_connection(
            ctx,
            to_node_id,
            to_socket_id,
            from_node_id,
            from_socket_id,
            edge_kind,
        )
        .await?;

        Ok(Connection::from_edge(&edge))
    }

    pub async fn new_to_parent(
        ctx: &DalContext,
        child_node_id: NodeId,
        parent_node_id: NodeId,
    ) -> DiagramResult<Self> {
        // TODO check if child already has parent and block connection

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

        Self::new(
            ctx,
            child_node_id,
            *from_socket.id(),
            parent_node_id,
            *to_socket.id(),
            EdgeKind::Symbolic,
        )
        .await
    }

    pub fn from_edge(edge: &Edge) -> Self {
        Self {
            id: *edge.id(),
            classification: *edge.kind(),
            source: Vertex {
                node_id: edge.tail_node_id(),
                socket_id: edge.tail_socket_id(),
            },
            destination: Vertex {
                node_id: edge.head_node_id(),
                socket_id: edge.head_socket_id(),
            },
            created_by: None,
            deleted_by: None,
        }
    }

    pub async fn delete_for_edge(ctx: &DalContext, edge_id: EdgeId) -> DiagramResult<()> {
        let mut edge = Edge::get_by_id(ctx, &edge_id)
            .await?
            .ok_or(DiagramError::EdgeNotFound)?;
        edge.delete_and_propagate(ctx).await?;
        Ok(())
    }

    pub async fn restore_for_edge(ctx: &DalContext, edge_id: EdgeId) -> DiagramResult<()> {
        Edge::restore_by_id(ctx, edge_id).await?;
        Ok(())
    }
}
