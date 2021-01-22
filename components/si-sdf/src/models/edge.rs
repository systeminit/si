use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{NatsTxn, NatsTxnError, PgTxn};
use crate::models::{
    list_model, next_update_clock, ListReply, ModelError, OrderByDirection, PageToken, Query,
    SiStorable, UpdateClockError,
};

const EDGE_DIRECT_SUCCESSOR_EDGES_BY_NODE_ID: &str =
    include_str!("../data/queries/edge_direct_successor_edges_by_node_id.sql");
const EDGE_DIRECT_SUCCESSOR_EDGES_BY_OBJECT_ID: &str =
    include_str!("../data/queries/edge_direct_successor_edges_by_object_id.sql");
const EDGE_DIRECT_PREDECESSOR_EDGES_BY_NODE_ID: &str =
    include_str!("../data/queries/edge_direct_predecessor_edges_by_node_id.sql");
const EDGE_DIRECT_PREDECESSOR_EDGES_BY_OBJECT_ID: &str =
    include_str!("../data/queries/edge_direct_predecessor_edges_by_object_id.sql");
const EDGE_BY_KIND_AND_HEAD_OBJECT_ID_AND_TAIL_TYPE_NAME: &str =
    include_str!("../data/queries/edge_by_kind_and_head_object_id_and_tail_type_name.sql");

#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
}

pub type EdgeResult<T> = Result<T, EdgeError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReply {
    pub edge: Edge,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AllPredecessorsRequest {
    pub object_id: Option<String>,
    pub node_id: Option<String>,
    pub edge_kind: EdgeKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AllPredecessorsReply {
    pub edges: Vec<Edge>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AllSuccessorsRequest {
    pub object_id: Option<String>,
    pub node_id: Option<String>,
    pub edge_kind: EdgeKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AllSuccessorsReply {
    pub edges: Vec<Edge>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vertex {
    pub node_id: String,
    pub object_id: String,
    pub socket: String,
    pub type_name: String,
}

impl Vertex {
    pub fn new(
        node_id: impl Into<String>,
        object_id: impl Into<String>,
        socket: impl Into<String>,
        type_name: impl Into<String>,
    ) -> Vertex {
        let node_id = node_id.into();
        let object_id = object_id.into();
        let socket = socket.into();
        let type_name = type_name.into();
        Vertex {
            node_id,
            object_id,
            socket,
            type_name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EdgeKind {
    Configures,
    Includes,
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            EdgeKind::Configures => "configures".to_string(),
            EdgeKind::Includes => "includes".to_string(),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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
        let update_clock = next_update_clock(workspace_id).await?;
        let row = txn
            .query_one(
                "SELECT object FROM edge_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                &[
                    &head_vertex.node_id,
                    &head_vertex.object_id,
                    &head_vertex.socket,
                    &head_vertex.type_name,
                    &tail_vertex.node_id,
                    &tail_vertex.object_id,
                    &tail_vertex.socket,
                    &tail_vertex.type_name,
                    &kind.to_string(),
                    &bidirectional,
                    &workspace_id,
                    &update_clock.epoch,
                    &update_clock.update_count,
                ],
            )
            .await?;
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

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> EdgeResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "edges",
            tenant_id,
            query,
            page_size,
            order_by,
            order_by_direction,
            page_token,
        )
        .await?;
        Ok(reply)
    }

    pub async fn delete(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EdgeResult<()> {
        let _row = txn
            .query_one("SELECT edge_delete_v1($1)", &[&self.id])
            .await?;
        self.si_storable.deleted = true;
        nats.delete(&self).await?;
        Ok(())
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

    pub async fn by_kind_and_head_object_id_and_tail_type_name(
        txn: &PgTxn<'_>,
        edge_kind: &EdgeKind,
        head_object_id: impl AsRef<str>,
        tail_type_name: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_object_id = head_object_id.as_ref();
        let tail_type_name = tail_type_name.as_ref();

        let rows = txn
            .query(
                EDGE_BY_KIND_AND_HEAD_OBJECT_ID_AND_TAIL_TYPE_NAME,
                &[&edge_kind.to_string(), &head_object_id, &tail_type_name],
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
