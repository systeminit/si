use serde::{Deserialize, Serialize};

use crate::edge::{Edge, EdgeId, EdgeKind};

use crate::change_status::ChangeStatus;
use crate::diagram::node::HistoryEventMetadata;
use crate::diagram::DiagramResult;
use crate::socket::SocketId;
use crate::{
    node::NodeId, ActorView, Component, ComponentError, DalContext, DiagramError, HistoryActor,
    Socket, SocketArity, StandardModel, User,
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
        let to_socket = Socket::get_by_id(ctx, &to_socket_id)
            .await?
            .ok_or(DiagramError::SocketNotFound)?;
        if *to_socket.arity() == SocketArity::One {
            let component = Component::find_for_node(ctx, to_node_id)
                .await?
                .ok_or(ComponentError::NotFoundForNode(to_node_id))?;
            let edges = Edge::list_for_component(ctx, *component.id()).await?;
            for mut edge in edges {
                if edge.tail_socket_id() == to_socket_id {
                    edge.delete_and_propagate(ctx).await?;
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

    pub async fn list(ctx: &DalContext) -> DiagramResult<Vec<Self>> {
        let edges = Edge::list(ctx).await?;
        let connections = edges.iter().map(Self::from_edge).collect::<Vec<Self>>();
        Ok(connections)
    }

    pub fn from_edge(edge: &Edge) -> Self {
        Self {
            id: *edge.id(),
            classification: edge.kind().clone(),
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

    pub fn source(&self) -> (NodeId, SocketId) {
        (self.source.node_id, self.source.socket_id)
    }

    pub fn destination(&self) -> (NodeId, SocketId) {
        (self.destination.node_id, self.destination.socket_id)
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramEdgeView {
    id: String,
    from_node_id: String,
    from_socket_id: String,
    to_node_id: String,
    to_socket_id: String,
    change_status: ChangeStatus,
    created_info: Option<HistoryEventMetadata>,
    deleted_info: Option<HistoryEventMetadata>,
}

impl DiagramEdgeView {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl DiagramEdgeView {
    pub async fn set_actor_details(&mut self, ctx: &DalContext, edge: &Edge) -> DiagramResult<()> {
        if let Some(user_pk) = edge.creation_user_pk() {
            let history_actor = HistoryActor::User(*user_pk);
            let actor = ActorView::from_history_actor(ctx, history_actor).await?;
            self.created_info = Some(HistoryEventMetadata {
                actor,
                timestamp: edge.timestamp().created_at,
            })
        }

        if let Some(user_pk) = edge.deletion_user_pk() {
            let history_actor = HistoryActor::User(*user_pk);
            let actor = ActorView::from_history_actor(ctx, history_actor).await?;
            self.deleted_info = Some(HistoryEventMetadata {
                actor,
                timestamp: ctx
                    .visibility()
                    .deleted_at
                    .ok_or(DiagramError::DeletionTimeStamp)?,
            })
        }

        Ok(())
    }

    pub fn from_with_change_status(conn: Connection, change_status: ChangeStatus) -> Self {
        Self {
            id: conn.id.to_string(),
            from_node_id: conn.source.node_id.to_string(),
            from_socket_id: conn.source.socket_id.to_string(),
            to_node_id: conn.destination.node_id.to_string(),
            to_socket_id: conn.destination.socket_id.to_string(),
            change_status,
            created_info: None,
            deleted_info: None,
        }
    }
}

impl From<Connection> for DiagramEdgeView {
    fn from(conn: Connection) -> Self {
        DiagramEdgeView::from_with_change_status(conn, ChangeStatus::Unmodified)
    }
}
