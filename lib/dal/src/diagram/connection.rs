use serde::{Deserialize, Serialize};

use crate::edge::{Edge, EdgeId, EdgeKind};

use crate::diagram::DiagramResult;
use crate::socket::SocketId;
use crate::{node::NodeId, DalContext, StandardModel};

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
}

impl Connection {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        from_node_id: NodeId,
        from_socket_id: SocketId,
        to_node_id: NodeId,
        to_socket_id: SocketId,
    ) -> DiagramResult<Self> {
        let edge =
            Edge::new_for_connection(ctx, to_node_id, to_socket_id, from_node_id, from_socket_id)
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
        }
    }

    pub fn source(&self) -> (NodeId, SocketId) {
        (self.source.node_id, self.source.socket_id)
    }

    pub fn destination(&self) -> (NodeId, SocketId) {
        (self.destination.node_id, self.destination.socket_id)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramEdgeView {
    id: String,
    #[serde(rename = "type")]
    ty: Option<String>,
    name: Option<String>,
    from_node_id: String,
    from_socket_id: String,
    to_node_id: String,
    to_socket_id: String,
    is_bidirectional: Option<bool>,
}

impl DiagramEdgeView {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<Connection> for DiagramEdgeView {
    fn from(conn: Connection) -> Self {
        let source_node_id: i64 = conn.source.node_id.into();
        let source_socket_id: i64 = conn.source.socket_id.into();
        let destination_node_id: i64 = conn.destination.node_id.into();
        let destination_socket_id: i64 = conn.destination.socket_id.into();
        let connection_id: i64 = conn.id.into();
        Self {
            id: connection_id.to_string(),
            ty: None,
            name: None,
            from_node_id: source_node_id.to_string(),
            from_socket_id: source_socket_id.to_string(),
            to_node_id: destination_node_id.to_string(),
            to_socket_id: destination_socket_id.to_string(),
            is_bidirectional: Some(false),
        }
    }
}
