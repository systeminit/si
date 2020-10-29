use crossbeam::queue::SegQueue;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::trace;

use crate::data::{Connection, Db};
use crate::models::{delete_model, insert_model, ModelError, SiStorable, SiStorableError};

#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("crossbeam pop error")]
    Crossbeam(String),
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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
    #[tracing::instrument(level = "trace")]
    pub async fn new(
        db: &Db,
        nats: &Connection,
        tail_vertex: Vertex,
        head_vertex: Vertex,
        bidirectional: bool,
        kind: EdgeKind,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: Option<String>,
    ) -> EdgeResult<Edge> {
        let si_storable = SiStorable::new(
            db,
            "edge",
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let id = si_storable.object_id.clone();
        let edge = Edge {
            id,
            head_vertex,
            tail_vertex,
            bidirectional,
            kind,
            si_storable,
        };
        insert_model(db, nats, &edge.id, &edge).await?;

        Ok(edge)
    }

    pub async fn all_successor_edges_by_node_id(
        db: &Db,
        edge_kind: EdgeKind,
        tail_vertex_node_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let tail_vertex_node_id = tail_vertex_node_id.into();

        let mut results: Vec<Edge> = Vec::new();

        let vertexes_to_check = SegQueue::<String>::new();
        vertexes_to_check.push(tail_vertex_node_id);

        while !vertexes_to_check.is_empty() {
            let tail_node_id = vertexes_to_check
                .pop()
                .map_err(|e| EdgeError::Crossbeam(e.to_string()))?;
            let query = format!(
                "SELECT a.*
                   FROM `{bucket}` AS a
                  WHERE a.siStorable.typeName = \"edge\"
                    AND a.kind = \"{edge_kind}\"
                    AND a.tailVertex.nodeId = \"{node_id}\"
                ",
                bucket = db.bucket_name,
                edge_kind = edge_kind,
                node_id = tail_node_id,
            );
            let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
            for qedge in query_results.into_iter() {
                vertexes_to_check.push(qedge.head_vertex.node_id.clone());
                results.push(qedge);
            }
        }

        Ok(results)
    }

    pub async fn all_successor_edges_by_object_id(
        db: &Db,
        edge_kind: EdgeKind,
        tail_vertex_object_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let tail_vertex_object_id = tail_vertex_object_id.into();

        let mut results: Vec<Edge> = Vec::new();

        let vertexes_to_check = SegQueue::<String>::new();
        vertexes_to_check.push(tail_vertex_object_id);

        while !vertexes_to_check.is_empty() {
            let tail_object_id = vertexes_to_check
                .pop()
                .map_err(|e| EdgeError::Crossbeam(e.to_string()))?;
            let query = format!(
                "SELECT a.*
                   FROM `{bucket}` AS a
                  WHERE a.siStorable.typeName = \"edge\"
                    AND a.kind = \"{edge_kind}\"
                    AND a.tailVertex.objectId = \"{object_id}\"
                ",
                bucket = db.bucket_name,
                edge_kind = edge_kind,
                object_id = tail_object_id,
            );
            let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
            for qedge in query_results.into_iter() {
                vertexes_to_check.push(qedge.head_vertex.object_id.clone());
                results.push(qedge);
            }
        }

        Ok(results)
    }

    pub async fn direct_successor_edges_by_object_id(
        db: &Db,
        edge_kind: EdgeKind,
        tail_vertex_object_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let tail_vertex_object_id = tail_vertex_object_id.into();

        let mut results: Vec<Edge> = Vec::new();

        let vertexes_to_check = SegQueue::<String>::new();
        vertexes_to_check.push(tail_vertex_object_id);

        while !vertexes_to_check.is_empty() {
            let head_object_id = vertexes_to_check
                .pop()
                .map_err(|e| EdgeError::Crossbeam(e.to_string()))?;
            let query = format!(
                "SELECT a.*
                   FROM `{bucket}` AS a
                  WHERE a.siStorable.typeName = \"edge\"
                    AND a.kind = \"{edge_kind}\"
                    AND a.tailVertex.objectId = \"{object_id}\"
                ",
                bucket = db.bucket_name,
                edge_kind = edge_kind,
                object_id = head_object_id,
            );
            let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
            for qedge in query_results.into_iter() {
                results.push(qedge);
            }
        }

        Ok(results)
    }

    pub async fn direct_successor_edges_by_node_id(
        db: &Db,
        edge_kind: EdgeKind,
        tail_vertex_node_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let tail_vertex_node_id = tail_vertex_node_id.into();

        let mut results: Vec<Edge> = Vec::new();

        let vertexes_to_check = SegQueue::<String>::new();
        vertexes_to_check.push(tail_vertex_node_id);

        while !vertexes_to_check.is_empty() {
            let head_node_id = vertexes_to_check
                .pop()
                .map_err(|e| EdgeError::Crossbeam(e.to_string()))?;
            let query = format!(
                "SELECT a.*
                   FROM `{bucket}` AS a
                  WHERE a.siStorable.typeName = \"edge\"
                    AND a.kind = \"{edge_kind}\"
                    AND a.tailVertex.nodeId = \"{node_id}\"
                ",
                bucket = db.bucket_name,
                edge_kind = edge_kind,
                node_id = head_node_id,
            );
            let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
            for qedge in query_results.into_iter() {
                results.push(qedge);
            }
        }

        Ok(results)
    }

    pub async fn direct_predecessor_edges_by_node_id(
        db: &Db,
        edge_kind: EdgeKind,
        head_vertex_node_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_vertex_node_id = head_vertex_node_id.into();

        let mut results: Vec<Edge> = Vec::new();

        let vertexes_to_check = SegQueue::<String>::new();
        vertexes_to_check.push(head_vertex_node_id);

        while !vertexes_to_check.is_empty() {
            let head_node_id = vertexes_to_check
                .pop()
                .map_err(|e| EdgeError::Crossbeam(e.to_string()))?;
            let query = format!(
                "SELECT a.*
                   FROM `{bucket}` AS a
                  WHERE a.siStorable.typeName = \"edge\"
                    AND a.kind = \"{edge_kind}\"
                    AND a.headVertex.nodeId = \"{node_id}\"
                ",
                bucket = db.bucket_name,
                edge_kind = edge_kind,
                node_id = head_node_id,
            );
            let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
            for qedge in query_results.into_iter() {
                results.push(qedge);
            }
        }

        Ok(results)
    }

    pub async fn all_predecessor_edges_by_node_id(
        db: &Db,
        edge_kind: EdgeKind,
        head_vertex_node_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_vertex_node_id = head_vertex_node_id.into();

        let mut results: Vec<Edge> = Vec::new();

        let vertexes_to_check = SegQueue::<String>::new();
        vertexes_to_check.push(head_vertex_node_id);

        while !vertexes_to_check.is_empty() {
            let head_node_id = vertexes_to_check
                .pop()
                .map_err(|e| EdgeError::Crossbeam(e.to_string()))?;
            let query = format!(
                "SELECT a.*
                   FROM `{bucket}` AS a
                  WHERE a.siStorable.typeName = \"edge\"
                    AND a.kind = \"{edge_kind}\"
                    AND a.headVertex.nodeId = \"{node_id}\"
                ",
                bucket = db.bucket_name,
                edge_kind = edge_kind,
                node_id = head_node_id,
            );
            let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
            for qedge in query_results.into_iter() {
                vertexes_to_check.push(qedge.tail_vertex.node_id.clone());
                results.push(qedge);
            }
        }

        Ok(results)
    }

    pub async fn all_predecessor_edges_by_object_id(
        db: &Db,
        edge_kind: EdgeKind,
        head_vertex_object_id: impl Into<String>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_vertex_object_id = head_vertex_object_id.into();

        let mut results: Vec<Edge> = Vec::new();

        let vertexes_to_check = SegQueue::<String>::new();
        vertexes_to_check.push(head_vertex_object_id);

        while !vertexes_to_check.is_empty() {
            let head_object_id = vertexes_to_check
                .pop()
                .map_err(|e| EdgeError::Crossbeam(e.to_string()))?;
            let query = format!(
                "SELECT a.*
                   FROM `{bucket}` AS a
                  WHERE a.siStorable.typeName = \"edge\"
                    AND a.kind = \"{edge_kind}\"
                    AND a.headVertex.objectId = \"{object_id}\"
                ",
                bucket = db.bucket_name,
                edge_kind = edge_kind,
                object_id = head_object_id,
            );
            let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
            for qedge in query_results.into_iter() {
                vertexes_to_check.push(qedge.tail_vertex.object_id.clone());
                results.push(qedge);
            }
        }

        Ok(results)
    }

    pub async fn by_kind_and_head_node_id(
        db: &Db,
        edge_kind: EdgeKind,
        head_node_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_node_id = head_node_id.as_ref();

        let query = format!(
            "SELECT a.*
           FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"edge\"
            AND a.kind = \"{edge_kind}\"
            AND a.headVertex.nodeId = \"{node_id}\"
        ",
            bucket = db.bucket_name,
            edge_kind = edge_kind,
            node_id = head_node_id,
        );
        let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
        Ok(query_results)
    }

    pub async fn by_kind_and_head_object_id(
        db: &Db,
        edge_kind: EdgeKind,
        head_object_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_object_id = head_object_id.as_ref();

        let query = format!(
            "SELECT a.*
           FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"edge\"
            AND a.kind = \"{edge_kind}\"
            AND a.headVertex.objectId = \"{object_id}\"
        ",
            bucket = db.bucket_name,
            edge_kind = edge_kind,
            object_id = head_object_id,
        );
        let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
        Ok(query_results)
    }

    pub async fn by_kind_and_head_object_id_and_tail_type_name(
        db: &Db,
        edge_kind: EdgeKind,
        head_object_id: impl AsRef<str>,
        tail_type_name: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let head_object_id = head_object_id.as_ref();
        let tail_type_name = tail_type_name.as_ref();

        let query = format!(
            "SELECT a.*
           FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"edge\"
            AND a.kind = \"{edge_kind}\"
            AND a.headVertex.objectId = \"{object_id}\"
            AND a.tailVertex.typeName = \"{type_name}\"
        ",
            bucket = db.bucket_name,
            edge_kind = edge_kind,
            object_id = head_object_id,
            type_name = tail_type_name,
        );
        trace!(?query, "edge query");
        let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
        Ok(query_results)
    }

    pub async fn by_kind_and_tail_node_id(
        db: &Db,
        edge_kind: EdgeKind,
        tail_node_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let tail_node_id = tail_node_id.as_ref();

        let query = format!(
            "SELECT a.*
           FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"edge\"
            AND a.kind = \"{edge_kind}\"
            AND a.tailVertex.nodeId = \"{node_id}\"
        ",
            bucket = db.bucket_name,
            edge_kind = edge_kind,
            node_id = tail_node_id,
        );
        let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
        Ok(query_results)
    }

    pub async fn by_kind_and_tail_object_id(
        db: &Db,
        edge_kind: EdgeKind,
        tail_object_id: impl AsRef<str>,
    ) -> EdgeResult<Vec<Edge>> {
        let tail_object_id = tail_object_id.as_ref();

        let query = format!(
            "SELECT a.*
           FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"edge\"
            AND a.kind = \"{edge_kind}\"
            AND a.tailVertex.objectId = \"{object_id}\"
        ",
            bucket = db.bucket_name,
            edge_kind = edge_kind,
            object_id = tail_object_id,
        );
        let query_results: Vec<Edge> = db.query_consistent(query, None).await?;
        Ok(query_results)
    }

    pub async fn delete(&self, db: &Db, nats: &Connection) -> EdgeResult<()> {
        delete_model(db, nats, &self.id, self).await?;
        Ok(())
    }
}
