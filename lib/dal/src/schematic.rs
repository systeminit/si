use crate::edge::{Edge, EdgeId, EdgeKind, VertexObjectKind};
use crate::{
    node::NodeId, node::NodeKindWithBaggage, ComponentError, EdgeError, HistoryActor, Node,
    NodeError, NodeKind, NodePosition, NodePositionError, NodeTemplate, NodeView, ReadTenancy,
    ReadTenancyError, StandardModel, StandardModelError, SystemError, SystemId, Visibility,
    WriteTenancy,
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
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
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
    #[error("component name not found")]
    ComponentNameNotFound,
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
    /// Only shows SchemaKind::Concrete and SchemaKind::Implementation
    /// They all have implementation input socket tied to a service output socket (?)
    Component,
    /// Only shows SchemaKind::Concept
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
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        head_node_id: &NodeId,
        head_socket_id: &SocketId,
        tail_node_id: &NodeId,
        tail_socket_id: &SocketId,
    ) -> SchematicResult<Self> {
        let head_node = Node::get_by_id(txn, &write_tenancy.into(), visibility, head_node_id)
            .await?
            .ok_or(SchematicError::Node(NodeError::NotFound(*head_node_id)))?;
        let tail_node = Node::get_by_id(txn, &write_tenancy.into(), visibility, tail_node_id)
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
            write_tenancy,
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

        // TODO: do we have to call Component::resolve_attribute for the head_component here?

        Ok(Self::from_edge(&edge))
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        read_tenancy: &ReadTenancy,
        visibility: &Visibility,
    ) -> SchematicResult<Vec<Self>> {
        let edges = Edge::list(txn, &read_tenancy.into(), visibility).await?;
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
        read_tenancy: &ReadTenancy,
        visibility: &Visibility,
        system_id: Option<SystemId>,
        root_node_id: NodeId,
    ) -> SchematicResult<Self> {
        let nodes: Vec<Node> = Node::list(txn, &read_tenancy.into(), visibility).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in nodes {
            let schematic_kind = (*node.kind()).into();
            let (schema, kind, name) = match node.kind() {
                NodeKind::Deployment | NodeKind::Component => {
                    let component = node
                        .component(txn, visibility)
                        .await?
                        .ok_or(SchematicError::ComponentNotFound)?;
                    let schema = component
                        .schema_with_tenancy(txn, &read_tenancy.into(), visibility)
                        .await?
                        .ok_or(SchematicError::SchemaNotFound)?;

                    (
                        schema,
                        match node.kind() {
                            NodeKind::Deployment => NodeKindWithBaggage::Deployment {
                                component_id: *component.id(),
                            },
                            NodeKind::Component => NodeKindWithBaggage::Component {
                                component_id: *component.id(),
                            },
                            NodeKind::System => unreachable!(),
                        },
                        component
                            .find_value_by_json_pointer::<String>(
                                txn,
                                read_tenancy,
                                visibility,
                                "/root/si/name",
                            )
                            .await?
                            .ok_or(SchematicError::ComponentNameNotFound)?,
                    )
                }
                NodeKind::System => {
                    // We're going to skip all `NodeKind::System` nodes
                    continue;

                    // TODO(fnichol): We were failing in `node.system()` with an `Error: dal
                    // schematic error: system not found` error. For the moment we're going to
                    // filter out system-backed nodes, but ultimately we might want to return all
                    // node kinds back to the frontend for use.
                    //
                    // let system = node
                    //     .system(txn, visibility)
                    //     .await?
                    //     .ok_or(SchematicError::SystemNotFound)?;
                    // let mut tenancy = tenancy.clone();
                    // tenancy.universal = true;
                    // let schema = system
                    //     .schema_with_tenancy(txn, &tenancy, visibility)
                    //     .await?
                    //     .ok_or(SchematicError::SchemaNotFound)?;

                    // (schema, system.name().to_owned())
                }
            };

            let position = NodePosition::find_by_node_id(
                txn,
                read_tenancy,
                visibility,
                schematic_kind,
                &system_id,
                root_node_id,
                *node.id(),
            )
            .await?;
            let template =
                NodeTemplate::new_from_schema_id(txn, read_tenancy, visibility, *schema.id())
                    .await?;
            let view = NodeView::new(
                name,
                node,
                kind,
                position.map_or(vec![], |p| vec![p]),
                template,
            );
            node_views.push(view);
        }

        let connections = Connection::list(txn, read_tenancy, visibility).await?;
        Ok(Self {
            nodes: node_views,
            connections,
        })
    }

    pub fn nodes(&self) -> &[NodeView] {
        &self.nodes
    }
}
