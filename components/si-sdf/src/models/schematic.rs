use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::{
    data::{DataError, PgTxn},
    models::{Edge, EdgeError, EdgeKind, Entity, EntityError, ModelError, Node, NodeError},
};

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("data layer error: {0}")]
    Data(#[from] DataError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
}

pub type SchematicResult<T> = Result<T, SchematicError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
enum SchematicKind {
    System,
    Deployment,
    Implementation,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ConnectionEdge {
    edge_id: String,
    node_id: String,
    socket_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Connections {
    predecessors: HashMap<String, Vec<ConnectionEdge>>,
    successors: HashMap<String, Vec<ConnectionEdge>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
enum SocketKind {
    Input,
    Output,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
enum SocketType {
    Object,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Socket {
    id: String,
    socket_kind: SocketKind,
    socket_type: SocketType,
    object_type: Option<String>,
}

// Translating from schematic node sockets from the typescript model
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SchematicNodeSockets {
    inputs: Vec<Socket>,
    outputs: Vec<Socket>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SchematicNode {
    node: Node,
    sockets: SchematicNodeSockets,
    object: serde_json::Value,
    connections: Connections,
}

impl SchematicNode {
    fn new(node: Node, object: serde_json::Value) -> SchematicNode {
        let sockets = SchematicNodeSockets {
            inputs: Vec::new(),
            outputs: Vec::new(),
        };
        let connections = Connections {
            predecessors: HashMap::new(),
            successors: HashMap::new(),
        };
        SchematicNode {
            node,
            object,
            sockets,
            connections,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schematic {
    // nodeId -> schematicNode
    nodes: HashMap<String, SchematicNode>,
    // edgeKind -> Edges
    edges: HashMap<String, Edge>,
}

impl Schematic {
    pub async fn get(
        txn: &PgTxn<'_>,
        root_object_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
        change_set_id: Option<String>,
        edge_kinds: Vec<EdgeKind>,
    ) -> SchematicResult<Schematic> {
        // Get the root object
        // Get its descendent edges
        // Populate the data
        // Profit!
        let root_object_id = root_object_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let root_entity = if let Some(change_set_id) = change_set_id.as_ref() {
            Entity::get_projection_or_head(&txn, &root_object_id, change_set_id).await?
        } else {
            Entity::get_head(&txn, &root_object_id).await?
        };
        let root_node = Node::get(&txn, &root_entity.node_id).await?;

        let mut edges: HashMap<String, Edge> = HashMap::new();
        let mut nodes: HashMap<String, SchematicNode> = HashMap::new();

        // An edge is included only if the object it points to has a head or a projection for this
        // changeset - otherwise, it doesn't exist in the schematic!
        for edge_kind in edge_kinds.iter() {
            let successor_edges =
                Edge::all_successor_edges_by_node_id(&txn, edge_kind, &root_node.id).await?;
            for successor_edge in successor_edges.into_iter() {
                if successor_edge.si_storable.workspace_id != workspace_id {
                    continue;
                }
                let successor_entity_id = &successor_edge.head_vertex.object_id;
                let relevant_successor_object = Entity::get_relevant_projection_or_head(
                    &txn,
                    successor_entity_id,
                    change_set_id.clone(),
                )
                .await?;
                if let Some(successor_entity) = relevant_successor_object {
                    edges.insert(successor_edge.id.clone(), successor_edge.clone());
                    let successor_node = Node::get(&txn, &successor_entity.node_id).await?;
                    let schematic_node =
                        if let Some(schematic_node) = nodes.get_mut(&successor_node.id) {
                            schematic_node
                        } else {
                            let successor_node_id = successor_node.id.clone();
                            let sn = SchematicNode::new(
                                successor_node.clone(),
                                serde_json::json![successor_entity],
                            );
                            nodes.insert(successor_node_id.clone(), sn);
                            // You just inserted it.. so it's cool.
                            nodes.get_mut(&successor_node_id).unwrap()
                        };

                    // Add a predecessors entry for the edge that we're on
                    schematic_node
                        .connections
                        .predecessors
                        .entry(successor_edge.kind.to_string())
                        .and_modify(|p| {
                            let connection_edge = ConnectionEdge {
                                edge_id: successor_edge.id.clone(),
                                node_id: successor_edge.tail_vertex.node_id.clone(),
                                socket_id: successor_edge.tail_vertex.socket.clone(),
                            };
                            p.push(connection_edge);
                        })
                        .or_insert_with(|| {
                            let connection_edge = ConnectionEdge {
                                edge_id: successor_edge.id.clone(),
                                node_id: successor_edge.tail_vertex.node_id.clone(),
                                socket_id: successor_edge.tail_vertex.socket.clone(),
                            };
                            let p = vec![connection_edge];
                            p
                        });
                    if successor_edge.head_vertex.node_id != successor_node.id {
                        nodes
                            .entry(successor_edge.head_vertex.node_id.clone())
                            .and_modify(|ns| {
                                ns.connections
                                    .successors
                                    .entry(successor_edge.kind.to_string())
                                    .and_modify(|s| {
                                        let connection_edge = ConnectionEdge {
                                            edge_id: successor_edge.id.clone(),
                                            node_id: successor_edge.head_vertex.node_id.clone(),
                                            socket_id: successor_edge.head_vertex.socket.clone(),
                                        };
                                        s.push(connection_edge);
                                    })
                                    .or_insert_with(|| {
                                        let connection_edge = ConnectionEdge {
                                            edge_id: successor_edge.id.clone(),
                                            node_id: successor_edge.head_vertex.node_id.clone(),
                                            socket_id: successor_edge.head_vertex.socket.clone(),
                                        };
                                        let s = vec![connection_edge];
                                        s
                                    });
                            });
                    }
                }
            }
        }
        let schematic = Schematic { nodes, edges };
        Ok(schematic)
    }
}
