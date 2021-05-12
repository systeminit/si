use serde::{Deserialize, Serialize};
use si_data::PgTxn;
use thiserror::Error;

use crate::{Edge, EdgeError, EdgeKind, Entity, EntityError, ModelError, Node, NodeError};

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
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
    #[error("no change set provided when one was needed")]
    NoChangeSet,
    #[error("node is missing in calculated schematic edge set")]
    MissingNode,
}

pub type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionPoint {
    pub node_id: String,
    pub node_name: String,
    pub node_description: String,
    pub node_type: String,
    pub socket_name: String,
    pub socket_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub edge: Edge,
    pub kind: EdgeKind,
    pub source: ConnectionPoint,
    pub destination: ConnectionPoint,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Connections {
    pub inbound: Vec<Connection>,
    pub outbound: Vec<Connection>,
}

impl Connection {
    pub async fn connections_from_entity(
        entity_id: impl AsRef<str>,
        edge_kinds: Vec<EdgeKind>,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
        txn: &PgTxn<'_>,
    ) -> ConnectionResult<Connections> {
        let mut inbound_connections: Vec<Connection> = Vec::new();
        let mut outbound_connections: Vec<Connection> = Vec::new();

        for edge_kind in edge_kinds.iter() {
            let entity = Entity::for_head_or_change_set_or_edit_session(
                &txn,
                &entity_id,
                change_set_id,
                edit_session_id,
            )
            .await?;

            let direct_edges =
                Edge::direct_edges_by_object_id(&txn, &edge_kind, entity.id.clone()).await?;

            for edge in direct_edges.predecessors.iter() {
                inbound_connections.push(
                    Connection::connection_from_edge(edge, change_set_id, edit_session_id, &txn)
                        .await?,
                );
            }

            for edge in direct_edges.successors.iter() {
                outbound_connections.push(
                    Connection::connection_from_edge(edge, change_set_id, edit_session_id, &txn)
                        .await?,
                );
            }
        }

        let connections = Connections {
            inbound: inbound_connections,
            outbound: outbound_connections,
        };

        Ok(connections)
    }

    pub async fn connection_from_edge(
        edge: &Edge,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
        txn: &PgTxn<'_>,
    ) -> ConnectionResult<Connection> {
        let source_node = Node::get(&txn, &edge.tail_vertex.node_id).await?;
        let souce_entity = Entity::for_head_or_change_set_or_edit_session(
            &txn,
            &edge.tail_vertex.object_id,
            change_set_id,
            edit_session_id,
        )
        .await?;

        let source = ConnectionPoint {
            node_id: source_node.id,
            node_name: souce_entity.name,
            node_description: souce_entity.description,
            node_type: souce_entity.entity_type,
            socket_name: edge.tail_vertex.socket.clone(),
            socket_type: String::from("output"),
        };

        let destination_node = Node::get(&txn, &edge.head_vertex.node_id).await?;
        let destination_entity = Entity::for_head_or_change_set_or_edit_session(
            &txn,
            &edge.head_vertex.object_id,
            change_set_id,
            edit_session_id,
        )
        .await?;

        let destination = ConnectionPoint {
            node_id: destination_node.id,
            node_name: destination_entity.name,
            node_description: destination_entity.description,
            node_type: destination_entity.entity_type,
            socket_name: edge.head_vertex.socket.clone(),
            socket_type: String::from("input"),
        };

        let connection = Connection {
            edge: edge.clone(),
            kind: edge.kind.clone(),
            source: source,
            destination: destination,
        };

        Ok(connection)
    }
}
