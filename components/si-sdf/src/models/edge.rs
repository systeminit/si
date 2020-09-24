use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{insert_model, ModelError, SiStorable, SiStorableError};

#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
}

pub type EdgeResult<T> = Result<T, EdgeError>;

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
pub struct EdgeSystemList {
    system_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EdgeKind {
    Configures(EdgeSystemList), // an array of systemIds this connection configures
    Includes,
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
}
