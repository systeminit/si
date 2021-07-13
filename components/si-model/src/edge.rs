use crate::{Entity, Node, SiStorable};
use serde::{Deserialize, Serialize};
use si_data::{pg::SqlState, NatsTxn, NatsTxnError, PgTxn};
use thiserror::Error;

const EDGE_DIRECT_SUCCESSOR_EDGES_BY_NODE_ID: &str =
    include_str!("./queries/edge_direct_successor_edges_by_node_id.sql");
const EDGE_DIRECT_SUCCESSOR_EDGES_BY_OBJECT_ID: &str =
    include_str!("./queries/edge_direct_successor_edges_by_object_id.sql");
const EDGE_DIRECT_PREDECESSOR_EDGES_BY_NODE_ID: &str =
    include_str!("./queries/edge_direct_predecessor_edges_by_node_id.sql");
const EDGE_DIRECT_PREDECESSOR_EDGES_BY_OBJECT_ID: &str =
    include_str!("./queries/edge_direct_predecessor_edges_by_object_id.sql");
const EDGE_BY_KIND_AND_HEAD_OBJECT_ID_AND_TAIL_OBJECT_TYPE: &str =
    include_str!("./queries/edge_by_kind_and_head_object_id_and_tail_object_type.sql");
const EDGE_BY_KIND_AND_OVERLAPPING_OBJECT_ID_SETS: &str =
    include_str!("./queries/edge_by_kind_and_overlapping_object_id_sets.sql");

#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("an edge with these vertexes and kind already exists")]
    EdgeExists,
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type EdgeResult<T> = Result<T, EdgeError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Edges {
    pub predecessors: Vec<Edge>,
    pub successors: Vec<Edge>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vertex {
    pub node_id: String,
    pub object_id: String,
    pub socket: String,
    pub object_type: String,
}

impl Vertex {
    pub fn new(
        node_id: impl Into<String>,
        object_id: impl Into<String>,
        socket: impl Into<String>,
        object_type: impl Into<String>,
    ) -> Vertex {
        let node_id = node_id.into();
        let object_id = object_id.into();
        let socket = socket.into();
        let object_type = object_type.into();
        Vertex {
            node_id,
            object_id,
            object_type,
            socket,
        }
    }

    pub fn from_node(node: &Node, socket: impl Into<String>) -> Vertex {
        let socket = socket.into();
        Vertex::new(&node.id, &node.object_id, socket, &node.object_type)
    }

    pub fn from_entity(entity: &Entity, socket: impl Into<String>) -> Vertex {
        let socket = socket.into();
        Vertex::new(&entity.node_id, &entity.id, socket, &entity.entity_type)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EdgeKind {
    Configures,
    Includes,
    Deployment,
    Component,
    Implementation,
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            EdgeKind::Configures => "configures".to_string(),
            EdgeKind::Includes => "includes".to_string(),
            EdgeKind::Deployment => "deployment".to_string(),
            EdgeKind::Component => "component".to_string(),
            EdgeKind::Implementation => "implementation".to_string(),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    pub tail_vertex: Vertex,
    pub head_vertex: Vertex,
    pub bidirectional: bool,
    pub kind: EdgeKind,
    pub si_storable: SiStorable,
}

impl Edge {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tail_vertex: Vertex,
        head_vertex: Vertex,
        bidirectional: bool,
        kind: EdgeKind,
        workspace_id: impl AsRef<str>,
    ) -> EdgeResult<Edge> {
        let workspace_id = workspace_id.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM edge_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    &head_vertex.node_id,
                    &head_vertex.object_id,
                    &head_vertex.socket,
                    &head_vertex.object_type,
                    &tail_vertex.node_id,
                    &tail_vertex.object_id,
                    &tail_vertex.socket,
                    &tail_vertex.object_type,
                    &kind.to_string(),
                    &bidirectional,
                    &workspace_id,
                ],
            )
            .await
            .map_err(|err| match err.code() {
                Some(sql_state) if sql_state == &SqlState::UNIQUE_VIOLATION => {
                    EdgeError::EdgeExists
                }
                _ => EdgeError::Pg(err),
            })?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Edge = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get(txn: &PgTxn<'_>, edge_id: impl AsRef<str>) -> EdgeResult<Edge> {
        let id = edge_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM edge_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn delete(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EdgeResult<()> {
        let _row = txn
            .query_one("SELECT edge_delete_v1($1)", &[&self.id])
            .await?;
        self.si_storable.deleted = true;
        nats.delete(&self).await?;
        Ok(())
    }

    pub async fn by_kind_and_overlapping_object_id_sets(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id_set: Vec<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let rows = txn
            .query(
                EDGE_BY_KIND_AND_OVERLAPPING_OBJECT_ID_SETS,
                &[&edge_kind.to_string(), &object_id_set],
            )
            .await?;

        let mut results: Vec<Edge> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let edge: Edge = serde_json::from_value(json)?;
            results.push(edge);
        }

        Ok(results)
    }

    pub async fn direct_successor_edges_by_node_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        node_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let node_id = node_id.as_ref();

        let rows = txn
            .query(
                EDGE_DIRECT_SUCCESSOR_EDGES_BY_NODE_ID,
                &[&edge_kind.to_string(), &node_id],
            )
            .await?;

        let mut results: Vec<Edge> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let edge: Edge = serde_json::from_value(json)?;
            results.push(edge);
        }

        Ok(results)
    }

    pub async fn all_successor_edges_by_node_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        node_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let node_id = node_id.into();

        let mut vertexes_to_check = vec![node_id];

        let mut results: Vec<Edge> = Vec::new();
        while let Some(head_node_id) = vertexes_to_check.pop() {
            let mut direct_results =
                Self::direct_successor_edges_by_node_id(txn, edge_kind, head_node_id).await?;

            for r in direct_results.iter() {
                vertexes_to_check.push(r.head_vertex.node_id.clone());
            }
            results.append(&mut direct_results);
        }

        Ok(results)
    }

    pub async fn all_successor_edges_by_node_id_for_edge_kinds(
        txn: &PgTxn<'_>,
        edge_kinds: &Vec<EdgeKind>,
        node_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let node_id = node_id.into();

        let mut vertexes_to_check = vec![node_id];

        let mut results: Vec<Edge> = Vec::new();
        while let Some(head_node_id) = vertexes_to_check.pop() {
            for edge_kind in edge_kinds.iter() {
                let mut direct_results =
                    Self::direct_successor_edges_by_node_id(txn, edge_kind, &head_node_id).await?;

                for r in direct_results.iter() {
                    vertexes_to_check.push(r.head_vertex.node_id.clone());
                }
                results.append(&mut direct_results);
            }
        }

        Ok(results)
    }

    pub async fn direct_successor_edges_by_object_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let object_id = object_id.as_ref();

        let rows = txn
            .query(
                EDGE_DIRECT_SUCCESSOR_EDGES_BY_OBJECT_ID,
                &[&edge_kind.to_string(), &object_id],
            )
            .await?;

        let mut results: Vec<Edge> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let edge: Edge = serde_json::from_value(json)?;
            results.push(edge);
        }

        Ok(results)
    }

    pub async fn all_successor_edges_by_object_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let object_id = object_id.into();

        let mut vertexes_to_check = vec![object_id];

        let mut results: Vec<Edge> = Vec::new();
        while let Some(head_object_id) = vertexes_to_check.pop() {
            let mut direct_results =
                Self::direct_successor_edges_by_object_id(txn, edge_kind, head_object_id).await?;

            for r in direct_results.iter() {
                vertexes_to_check.push(r.head_vertex.object_id.clone());
            }
            results.append(&mut direct_results);
        }

        Ok(results)
    }

    pub async fn direct_predecessor_edges_by_node_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        node_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let node_id = node_id.as_ref();

        let rows = txn
            .query(
                EDGE_DIRECT_PREDECESSOR_EDGES_BY_NODE_ID,
                &[&edge_kind.to_string(), &node_id],
            )
            .await?;

        let mut results: Vec<Edge> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let edge: Edge = serde_json::from_value(json)?;
            results.push(edge);
        }

        Ok(results)
    }

    pub async fn all_predecessor_edges_by_node_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        node_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let node_id = node_id.into();

        let mut vertexes_to_check = vec![node_id];

        let mut results: Vec<Edge> = Vec::new();
        while let Some(tail_node_id) = vertexes_to_check.pop() {
            let mut direct_results =
                Self::direct_predecessor_edges_by_node_id(txn, edge_kind, tail_node_id).await?;

            for r in direct_results.iter() {
                vertexes_to_check.push(r.tail_vertex.node_id.clone());
            }
            results.append(&mut direct_results);
        }

        Ok(results)
    }

    pub async fn direct_predecessor_edges_by_object_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let object_id = object_id.as_ref();

        let rows = txn
            .query(
                EDGE_DIRECT_PREDECESSOR_EDGES_BY_OBJECT_ID,
                &[&edge_kind.to_string(), &object_id],
            )
            .await?;

        let mut results: Vec<Edge> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let edge: Edge = serde_json::from_value(json)?;
            results.push(edge);
        }

        Ok(results)
    }

    pub async fn all_predecessor_edges_by_object_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let object_id = object_id.into();

        let mut vertexes_to_check = vec![object_id];

        let mut results: Vec<Edge> = Vec::new();
        while let Some(tail_object_id) = vertexes_to_check.pop() {
            let mut direct_results =
                Self::direct_predecessor_edges_by_object_id(txn, edge_kind, tail_object_id).await?;

            for r in direct_results.iter() {
                vertexes_to_check.push(r.tail_vertex.object_id.clone());
            }
            results.append(&mut direct_results);
        }

        Ok(results)
    }

    pub async fn direct_edges_by_object_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id: impl Into<String>,
    ) -> EdgeResult<Edges> {
        let object_id = object_id.into();

        let mut direct_predecessors: Vec<Edge> = Vec::new();
        let direct_predecessor_edges =
            Edge::direct_predecessor_edges_by_object_id(txn, edge_kind, object_id.clone()).await?;
        for edge in direct_predecessor_edges.into_iter() {
            direct_predecessors.push(edge.clone());
        }

        let mut direct_successors: Vec<Edge> = Vec::new();
        let direct_successor_edges =
            Edge::direct_successor_edges_by_object_id(txn, edge_kind, object_id).await?;
        for edge in direct_successor_edges.into_iter() {
            direct_successors.push(edge.clone());
        }

        let results = Edges {
            predecessors: direct_predecessors,
            successors: direct_successors,
        };

        Ok(results)
    }

    pub async fn direct_edges_by_node_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        node_id: impl Into<String>,
    ) -> EdgeResult<Edges> {
        let node_id = node_id.into();

        let mut direct_predecessors: Vec<Edge> = Vec::new();

        let direct_predecessor_edges =
            Edge::direct_predecessor_edges_by_node_id(txn, edge_kind, node_id.clone()).await?;
        for edge in direct_predecessor_edges.into_iter() {
            direct_predecessors.push(edge.clone());
        }

        let mut direct_successors: Vec<Edge> = Vec::new();
        let direct_successor_edges =
            Edge::direct_successor_edges_by_node_id(txn, edge_kind, node_id).await?;
        for edge in direct_successor_edges.into_iter() {
            direct_successors.push(edge.clone());
        }

        let results = Edges {
            predecessors: direct_predecessors,
            successors: direct_successors,
        };

        Ok(results)
    }

    // TODO(fnichol): the original implementation is precisely the same as
    // direct_predecessor_edges_by_node_id, so this *should* be able to go away, after
    // refactoring the call sites...
    pub async fn by_kind_and_head_node_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        node_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        Self::direct_predecessor_edges_by_node_id(txn, edge_kind, node_id).await
    }

    // TODO(fnichol): the original implementation is precisely the same as
    // direct_predecessor_edges_by_object_id, so this *should* be able to go away, after
    // refactoring the call sites...
    pub async fn by_kind_and_head_object_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        Self::direct_predecessor_edges_by_object_id(txn, edge_kind, object_id.as_ref()).await
    }

    pub async fn by_kind_and_tail_node_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        node_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        Self::direct_successor_edges_by_node_id(txn, edge_kind, node_id).await
    }

    pub async fn by_kind_and_tail_object_id(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        object_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        Self::direct_successor_edges_by_object_id(txn, edge_kind, object_id).await
    }

    pub async fn by_kind_and_head_object_id_and_tail_object_type(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        head_object_id: impl AsRef<str>,
        tail_object_type: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_object_id = head_object_id.as_ref();
        let tail_object_type = tail_object_type.as_ref();

        let rows = txn
            .query(
                EDGE_BY_KIND_AND_HEAD_OBJECT_ID_AND_TAIL_OBJECT_TYPE,
                &[&edge_kind.to_string(), &head_object_id, &tail_object_type],
            )
            .await?;

        let mut results: Vec<Edge> = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let edge: Edge = serde_json::from_value(json)?;
            results.push(edge);
        }

        Ok(results)
    }
}
