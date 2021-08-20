use crate::{
    workflow::WorkflowRunListItem, Edge, EdgeError, EdgeKind, Entity, EntityError, ModelError,
    Node, NodeError, NodePosition, NodePositionError, Qualification, QualificationError, Resource,
    ResourceError, SiStorable, WorkflowError, WorkflowRun,
};
use serde::{Deserialize, Serialize};
use si_data::PgTxn;
use std::collections::HashMap;
use thiserror::Error;

const ENTITY_FOR_LINK_MENU: &str = include_str!("./queries/entity_for_link_menu.sql");

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("node is missing in calculated schematic edge set")]
    MissingNode,
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("no change set provided when one was needed")]
    NoChangeSet,
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("qualification error: {0}")]
    Qualification(#[from] QualificationError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("workflow error: {0}")]
    Workflow(#[from] WorkflowError),
}

pub type SchematicResult<T> = Result<T, SchematicError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SchematicKind {
    System,
    Deployment,
    Component,
    Implementation,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeWithPositions {
    pub id: String,
    pub object_type: String,
    pub object_id: String,
    pub positions: HashMap<String, NodePosition>,
    pub si_storable: SiStorable,
}

impl NodeWithPositions {
    pub async fn from_node_position(txn: &PgTxn<'_>, node: Node) -> SchematicResult<Self> {
        let mut positions = HashMap::new();
        for node_position in NodePosition::get_by_node_id(&txn, &node.id)
            .await?
            .into_iter()
        {
            positions.insert(node_position.context_id.clone(), node_position);
        }

        Ok(Self {
            id: node.id,
            object_type: node.object_type,
            object_id: node.object_id,
            positions,
            si_storable: node.si_storable,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionEdge {
    pub edge_id: String,
    pub node_id: String,
    pub socket_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Connections {
    pub predecessors: HashMap<String, Vec<ConnectionEdge>>,
    pub successors: HashMap<String, Vec<ConnectionEdge>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SocketKind {
    Input,
    Output,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SocketType {
    Object,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Socket {
    pub id: String,
    pub socket_kind: SocketKind,
    pub socket_type: SocketType,
    pub object_type: Option<String>,
}

// Translating from schematic node sockets from the typescript model
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchematicNodeSockets {
    pub inputs: Vec<Socket>,
    pub outputs: Vec<Socket>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchematicNode {
    pub node: NodeWithPositions,
    pub sockets: SchematicNodeSockets,
    pub object: Entity,
    pub connections: Connections,
    pub resources: HashMap<String, Resource>,
    pub qualifications: Vec<Qualification>,
    pub workflow_runs: HashMap<String, WorkflowRunListItem>,
}

impl SchematicNode {
    pub async fn new(
        txn: &PgTxn<'_>,
        node: Node,
        object: Entity,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
    ) -> SchematicResult<Self> {
        let node = NodeWithPositions::from_node_position(&txn, node).await?;

        let sockets = SchematicNodeSockets {
            inputs: Vec::new(),
            outputs: Vec::new(),
        };
        let connections = Connections {
            predecessors: HashMap::new(),
            successors: HashMap::new(),
        };

        let mut resources = HashMap::new();
        let resource_list = Resource::for_entity(&txn, &object.id).await?;
        let mut system_ids = Vec::new();
        for resource in resource_list.into_iter() {
            system_ids.push(resource.system_id.clone());
            resources.insert(resource.system_id.clone(), resource);
        }

        let qualification = Qualification::for_head_or_change_set_or_edit_session(
            txn,
            &object.id,
            change_set_id,
            edit_session_id,
        )
        .await?;

        let mut workflow_runs = HashMap::new();
        for system_id in system_ids.iter() {
            let action_name: Option<&str> = None;
            let workflow_run_items = WorkflowRun::list_actions_for_schematic(
                &txn,
                &object.id,
                &system_id,
                &object.si_storable.workspace_id,
                action_name,
            )
            .await?;
            if let Some(wr) = workflow_run_items.into_iter().nth(0) {
                workflow_runs.insert(system_id.clone(), wr);
            }
        }

        let sn = Self {
            node,
            object,
            sockets,
            connections,
            resources,
            qualifications: qualification,
            workflow_runs,
        };

        Ok(sn)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schematic {
    // nodeId -> schematicNode
    pub nodes: HashMap<String, SchematicNode>,
    // edgeId -> Edge
    pub edges: HashMap<String, Edge>,
}

impl Schematic {
    pub async fn get_by_schematic_kind(
        txn: &PgTxn<'_>,
        schematic_kind: &SchematicKind,
        root_object_id: impl AsRef<str>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
    ) -> SchematicResult<Schematic> {
        let root_object_id = root_object_id.as_ref();
        let mut schematic = match schematic_kind {
            SchematicKind::Deployment => {
                Schematic::get_deployment(&txn, root_object_id, change_set_id, edit_session_id)
                    .await?
            }

            SchematicKind::Component => {
                Schematic::get_component(&txn, root_object_id, change_set_id, edit_session_id)
                    .await?
            }

            _ => {
                Schematic::get(
                    &txn,
                    root_object_id,
                    change_set_id,
                    edit_session_id,
                    vec![
                        EdgeKind::Configures,
                        EdgeKind::Deployment,
                        EdgeKind::Implementation,
                    ],
                    // vec![EdgeKind::Configures, EdgeKind::Deployment, EdgeKind::Implementation],
                )
                .await?
            }
        };

        if schematic_kind == &SchematicKind::Deployment {
            let root_node = Node::get_for_object_id(&txn, &root_object_id, None).await?;
            schematic.prune_node(root_node.id);
        }

        return Ok(schematic);
    }

    pub async fn get_component(
        txn: &PgTxn<'_>,
        deployment_entity_id: impl AsRef<str>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
    ) -> SchematicResult<Schematic> {
        let deployment_entity_id = deployment_entity_id.as_ref();
        let deployment_entity = Entity::for_head_or_change_set_or_edit_session(
            &txn,
            &deployment_entity_id,
            change_set_id.as_ref(),
            edit_session_id.as_ref(),
        )
        .await?;
        let deployment_node = Node::get(&txn, &deployment_entity.node_id).await?;

        let mut edges: HashMap<String, Edge> = HashMap::new();
        let mut nodes: HashMap<String, SchematicNode> = HashMap::new();
        let mut object_id_set: Vec<String> = vec![deployment_entity_id.into()];

        let sn = SchematicNode::new(
            &txn,
            deployment_node.clone(),
            deployment_entity,
            change_set_id.as_ref(),
            edit_session_id.as_ref(),
        )
        .await?;
        nodes.insert(deployment_node.id.clone(), sn);

        let all_node_edges = Edge::direct_successor_edges_by_node_id(
            &txn,
            &EdgeKind::Component,
            &deployment_node.id,
        )
        .await?;

        for node_edge in all_node_edges.into_iter() {
            let successor_entity_id = &node_edge.head_vertex.object_id;
            object_id_set.push(successor_entity_id.clone());
            let successor_entity = match Entity::for_head_or_change_set_or_edit_session(
                &txn,
                successor_entity_id,
                change_set_id.as_ref(),
                edit_session_id.as_ref(),
            )
            .await
            {
                Ok(entity) => entity,
                Err(_e) => continue,
            };
            //edges.insert(node_edge.id.clone(), node_edge.clone());
            let successor_node = Node::get(&txn, &successor_entity.node_id).await?;
            if let None = nodes.get(&successor_node.id) {
                let successor_node_id = successor_node.id.clone();
                let sn = SchematicNode::new(
                    &txn,
                    successor_node.clone(),
                    successor_entity,
                    change_set_id.as_ref(),
                    edit_session_id.as_ref(),
                )
                .await?;
                nodes.insert(successor_node_id.clone(), sn);
            };
        }

        let full_edges = Edge::by_kind_and_overlapping_object_id_sets(
            &txn,
            &EdgeKind::Configures,
            object_id_set,
        )
        .await?;

        for successor_edge in full_edges.into_iter() {
            let successor_entity_id = &successor_edge.head_vertex.object_id;
            let successor_entity = match Entity::for_head_or_change_set_or_edit_session(
                &txn,
                successor_entity_id,
                change_set_id.as_ref(),
                edit_session_id.as_ref(),
            )
            .await
            {
                Ok(entity) => entity,
                Err(_e) => continue,
            };
            edges.insert(successor_edge.id.clone(), successor_edge.clone());
            let successor_node = Node::get(&txn, &successor_entity.node_id).await?;
            let schematic_node = nodes
                .get_mut(&successor_node.id)
                .ok_or(SchematicError::MissingNode)?;

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

        let schematic = Schematic { nodes, edges };
        Ok(schematic)
    }

    pub async fn get_deployment(
        txn: &PgTxn<'_>,
        application_entity_id: impl AsRef<str>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
    ) -> SchematicResult<Schematic> {
        let application_entity_id = application_entity_id.as_ref();
        let application_entity = Entity::for_head_or_change_set_or_edit_session(
            &txn,
            &application_entity_id,
            change_set_id.as_ref(),
            edit_session_id.as_ref(),
        )
        .await?;
        let application_node = Node::get(&txn, &application_entity.node_id).await?;

        let mut edges: HashMap<String, Edge> = HashMap::new();
        let mut nodes: HashMap<String, SchematicNode> = HashMap::new();
        let mut object_id_set: Vec<String> = vec![application_entity_id.into()];

        let sn = SchematicNode::new(
            &txn,
            application_node.clone(),
            application_entity,
            change_set_id.as_ref(),
            edit_session_id.as_ref(),
        )
        .await?;
        nodes.insert(application_node.id.clone(), sn);

        let all_node_edges = Edge::direct_successor_edges_by_node_id(
            &txn,
            &EdgeKind::Component,
            &application_node.id,
        )
        .await?;

        for node_edge in all_node_edges.into_iter() {
            let successor_entity_id = &node_edge.head_vertex.object_id;
            object_id_set.push(successor_entity_id.clone());
            let successor_entity = match Entity::for_head_or_change_set_or_edit_session(
                &txn,
                successor_entity_id,
                change_set_id.as_ref(),
                edit_session_id.as_ref(),
            )
            .await
            {
                Ok(entity) => entity,
                Err(_e) => continue,
            };
            //edges.insert(node_edge.id.clone(), node_edge.clone());
            let successor_node = Node::get(&txn, &successor_entity.node_id).await?;
            if let None = nodes.get(&successor_node.id) {
                let successor_node_id = successor_node.id.clone();
                let sn = SchematicNode::new(
                    &txn,
                    successor_node.clone(),
                    successor_entity,
                    change_set_id.as_ref(),
                    edit_session_id.as_ref(),
                )
                .await?;
                nodes.insert(successor_node_id.clone(), sn);
            };
        }

        let full_edges = Edge::by_kind_and_overlapping_object_id_sets(
            &txn,
            &EdgeKind::Deployment,
            object_id_set,
        )
        .await?;

        for successor_edge in full_edges.into_iter() {
            let successor_entity_id = &successor_edge.head_vertex.object_id;
            let successor_entity = match Entity::for_head_or_change_set_or_edit_session(
                &txn,
                successor_entity_id,
                change_set_id.as_ref(),
                edit_session_id.as_ref(),
            )
            .await
            {
                Ok(entity) => entity,
                Err(_e) => continue,
            };
            edges.insert(successor_edge.id.clone(), successor_edge.clone());
            let successor_node = Node::get(&txn, &successor_entity.node_id).await?;
            let schematic_node = nodes
                .get_mut(&successor_node.id)
                .ok_or(SchematicError::MissingNode)?;

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

        let schematic = Schematic { nodes, edges };
        Ok(schematic)
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        root_object_id: impl AsRef<str>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
        edge_kinds: Vec<EdgeKind>,
    ) -> SchematicResult<Schematic> {
        // Get the root object
        // Get its descendent edges
        // Populate the data
        // Profit!
        let root_object_id = root_object_id.as_ref();
        let root_entity = Entity::for_head_or_change_set_or_edit_session(
            &txn,
            &root_object_id,
            change_set_id.as_ref(),
            edit_session_id.as_ref(),
        )
        .await?;
        let root_node = Node::get(&txn, &root_entity.node_id).await?;

        let mut edges: HashMap<String, Edge> = HashMap::new();
        let mut nodes: HashMap<String, SchematicNode> = HashMap::new();

        let sn = SchematicNode::new(
            &txn,
            root_node.clone(),
            root_entity,
            change_set_id.as_ref(),
            edit_session_id.as_ref(),
        )
        .await?;
        nodes.insert(root_node.id.clone(), sn);

        // An edge is included only if the object it points to has a head or a projection for this
        // changeset, or edit session - otherwise, it doesn't exist in the schematic!
        let successor_edges =
            Edge::all_successor_edges_by_node_id_for_edge_kinds(&txn, &edge_kinds, &root_node.id)
                .await?;

        for successor_edge in successor_edges.into_iter() {
            let successor_entity_id = &successor_edge.head_vertex.object_id;
            let successor_entity = match Entity::for_head_or_change_set_or_edit_session(
                &txn,
                successor_entity_id,
                change_set_id.as_ref(),
                edit_session_id.as_ref(),
            )
            .await
            {
                Ok(entity) => entity,
                Err(_e) => continue,
            };
            edges.insert(successor_edge.id.clone(), successor_edge.clone());
            let successor_node = Node::get(&txn, &successor_entity.node_id).await?;
            let schematic_node = if let Some(schematic_node) = nodes.get_mut(&successor_node.id) {
                schematic_node
            } else {
                let successor_node_id = successor_node.id.clone();
                let sn = SchematicNode::new(
                    &txn,
                    successor_node.clone(),
                    successor_entity,
                    change_set_id.as_ref(),
                    edit_session_id.as_ref(),
                )
                .await?;
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

        let schematic = Schematic { nodes, edges };
        Ok(schematic)
    }

    pub fn prune_node(&mut self, prune_node_id: impl AsRef<str>) {
        let prune_node_id = prune_node_id.as_ref();

        // Remove any node entries with the prune node id
        self.nodes.retain(|key, _| key != prune_node_id);

        // Remove any successor/predecessor connections that refer to the prune node id
        for (_, node) in self.nodes.iter_mut() {
            for (_, connection_edges) in node.connections.predecessors.iter_mut() {
                // Remove any connection edges that refer to the prune node id
                connection_edges.retain(|connection_edge| connection_edge.node_id != prune_node_id);
            }
            // Remove any remaining empty arrays
            node.connections
                .predecessors
                .retain(|_, values| !values.is_empty());

            for (_, connection_edges) in node.connections.successors.iter_mut() {
                // Remove any connection edges that refer to the prune node id
                connection_edges.retain(|connection_edge| connection_edge.node_id != prune_node_id);
            }
            // Remove any remaining empty arrays
            node.connections
                .successors
                .retain(|_, values| !values.is_empty());
        }

        // Remove any edges whose tail or head vertex refers to the prune node id
        self.edges.retain(|_, edge| {
            edge.head_vertex.node_id != prune_node_id && edge.tail_vertex.node_id != prune_node_id
        });
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinkNodeItem {
    pub kind: String,
    pub entity_type: String,
    pub node_id: String,
    pub entity_id: String,
    pub name: String,
}

impl From<Entity> for LinkNodeItem {
    fn from(entity: Entity) -> Self {
        Self {
            kind: String::from("link"),
            entity_type: entity.entity_type,
            entity_id: entity.id,
            node_id: entity.node_id,
            name: entity.name,
        }
    }
}

pub async fn get_link_menu(
    txn: &PgTxn<'_>,
    workspace_id: impl AsRef<str>,
    change_set_id: impl AsRef<str>,
    edit_session_id: impl AsRef<str>,
    component_entity_id: impl AsRef<str>,
    entity_types: Vec<String>,
) -> SchematicResult<Vec<LinkNodeItem>> {
    let workspace_id = workspace_id.as_ref();
    let change_set_id = change_set_id.as_ref();
    let edit_session_id = edit_session_id.as_ref();
    let component_entity_id = component_entity_id.as_ref();

    let mut links: Vec<LinkNodeItem> = Vec::new();
    let rows = txn
        .query(
            ENTITY_FOR_LINK_MENU,
            &[
                &entity_types,
                &component_entity_id,
                &change_set_id,
                &edit_session_id,
                &workspace_id,
            ],
        )
        .await?;
    for row in rows.into_iter() {
        let entity_json: serde_json::Value = row.try_get("object")?;
        let entity: Entity = serde_json::from_value(entity_json)?;
        links.push(entity.into());
    }
    Ok(links)
}
