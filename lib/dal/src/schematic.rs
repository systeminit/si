use crate::edge::{Edge, EdgeId, EdgeKind, VertexObjectKind};
use crate::{
    node::NodeId, ComponentError, EdgeError, HistoryActor, Node, NodeError, NodeKind, NodePosition,
    NodePositionError, NodeTemplate, NodeView, StandardModel, StandardModelError, SystemError,
    SystemId, Tenancy, Visibility,
};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

use crate::socket::SocketId;
use si_data::{NatsTxn, PgError, PgTxn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("position not found")]
    PositionNotFound,
    #[error("component not found")]
    ComponentNotFound,
    #[error("schema not found")]
    SchemaNotFound,
    #[error("system error: {0}")]
    System(#[from] SystemError),
    #[error("system not found")]
    SystemNotFound,
}

pub type SchematicResult<T> = Result<T, SchematicError>;

#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SchematicKind {
    Component,
    Deployment,
    System,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    id: EdgeId,
    classification: EdgeKind,
    source: Vertex,
    destination: Vertex,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Vertex {
    node_id: NodeId,
    socket_id: SocketId,
}

impl Connection {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        head_node_id: &NodeId,
        head_socket_id: &SocketId,
        tail_node_id: &NodeId,
        tail_socket_id: &SocketId,
    ) -> SchematicResult<Self> {
        let head_node = Node::get_by_id(txn, tenancy, visibility, head_node_id)
            .await?
            .ok_or(SchematicError::Node(NodeError::NotFound(*head_node_id)))?;
        let tail_node = Node::get_by_id(txn, tenancy, visibility, tail_node_id)
            .await?
            .ok_or(SchematicError::Node(NodeError::NotFound(*tail_node_id)))?;

        let head_component = head_node
            .component(txn, visibility)
            .await?
            .ok_or(SchematicError::Node(NodeError::ComponentIsNone))?;
        let tail_component = tail_node
            .component(txn, visibility)
            .await?
            .ok_or(SchematicError::Node(NodeError::ComponentIsNone))?;

        // TODO(nick): a lot of hardcoded values here along with the (temporary) insinuation that an
        // edge is equivalent to a connection.
        let edge = match Edge::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            EdgeKind::Configures,
            *head_node_id,
            VertexObjectKind::Component,
            (*head_component.id()).into(),
            *head_socket_id,
            *tail_node_id,
            VertexObjectKind::Component,
            (*tail_component.id()).into(),
            *tail_socket_id,
        )
        .await
        {
            Ok(edge) => edge,
            Err(e) => return Err(SchematicError::Edge(e)),
        };

        Ok(Self::from_edge(&edge))
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
    ) -> SchematicResult<Vec<Self>> {
        let edges = Edge::list(txn, tenancy, visibility).await?;
        let connections = edges.iter().map(Self::from_edge).collect::<Vec<Self>>();
        Ok(connections)
    }

    // NOTE(nick): we clone kind for now, but I'd imagine Connection will have it's own "kind"
    // wrapper in the future.
    pub fn from_edge(edge: &Edge) -> Self {
        Self {
            id: *edge.id(),
            classification: edge.kind().clone(),
            source: Vertex {
                node_id: edge.head_node_id(),
                socket_id: edge.head_socket_id(),
            },
            destination: Vertex {
                node_id: edge.tail_node_id(),
                socket_id: edge.tail_socket_id(),
            },
        }
    }

    // NOTE(nick): value is moved, but that's fine for tests.
    pub fn source(&self) -> (NodeId, SocketId) {
        (self.source.node_id, self.source.socket_id)
    }

    // NOTE(nick): value is moved, but that's fine for tests.
    pub fn destination(&self) -> (NodeId, SocketId) {
        (self.destination.node_id, self.destination.socket_id)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Schematic {
    nodes: Vec<NodeView>,
    connections: Vec<Connection>,
}

impl Schematic {
    pub async fn find(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: Option<SystemId>,
        root_node_id: NodeId,
    ) -> SchematicResult<Self> {
        let nodes: Vec<Node> = Node::list(txn, tenancy, visibility).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in nodes {
            let (schema, name, schematic_kind) = match node.kind() {
                NodeKind::Component => {
                    let component = node
                        .component(txn, visibility)
                        .await?
                        .ok_or(SchematicError::ComponentNotFound)?;
                    let mut tenancy = tenancy.clone();
                    tenancy.universal = true;
                    let schema = component
                        .schema_with_tenancy(txn, &tenancy, visibility)
                        .await?
                        .ok_or(SchematicError::SchemaNotFound)?;
                    (
                        schema,
                        component.name().to_owned(),
                        SchematicKind::Component,
                    )
                }
                NodeKind::System => {
                    let system = node
                        .system(txn, visibility)
                        .await?
                        .ok_or(SchematicError::SystemNotFound)?;
                    let mut tenancy = tenancy.clone();
                    tenancy.universal = true;
                    let schema = system
                        .schema_with_tenancy(txn, &tenancy, visibility)
                        .await?
                        .ok_or(SchematicError::SchemaNotFound)?;

                    (schema, system.name().to_owned(), SchematicKind::System)
                }
            };

            let position = NodePosition::find_by_node_id(
                txn,
                tenancy,
                visibility,
                schematic_kind,
                &system_id,
                root_node_id,
                *node.id(),
            )
            .await?;
            let template =
                NodeTemplate::new_from_schema_id(txn, tenancy, visibility, *schema.id()).await?;
            let view = NodeView::new(name, node, position.map_or(vec![], |p| vec![p]), template);
            node_views.push(view);
        }

        let connections = Connection::list(txn, tenancy, visibility).await?;
        Ok(Self {
            nodes: node_views,
            connections,
        })
    }

    pub fn nodes(&self) -> &[NodeView] {
        &self.nodes
    }
}
