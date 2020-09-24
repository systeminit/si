use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{
    get_model, insert_model, Edge, EdgeError, EdgeKind, EdgeSystemList, Entity, EntityError,
    ModelError, OpReply, OpRequest, SiChangeSetError, SiStorable, SiStorableError, System,
    SystemError, Vertex,
};

use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error with linked entity: {0}")]
    Entity(#[from] EntityError),
    #[error("error with linked system: {0}")]
    System(#[from] SystemError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("no head object found; logic error")]
    NoHead,
    #[error("no projection object found")]
    NoProjection,
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("no object id; bug!")]
    NoObjectId,
}

pub type NodeResult<T> = Result<T, NodeError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
    pub kind: NodeKind,
    pub object_type: String,
    pub organization_id: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: Node,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchIncludeSystemRequest {
    pub system_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchIncludeSystemReply {
    pub edge: Edge,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchOp {
    IncludeSystem(PatchIncludeSystemRequest),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchReply {
    IncludeSystem(PatchIncludeSystemReply),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchRequest {
    pub op: PatchOp,
    pub organization_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPatchRequest {
    pub op: OpRequest,
    pub organization_id: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ObjectPatchReply {
    Op(OpReply),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Position {
    x: u64,
    y: u64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Entity,
    System,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node_kind_string = match self {
            NodeKind::System => "system",
            NodeKind::Entity => "entity",
        };
        write!(f, "{}", node_kind_string)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub positions: HashMap<String, Position>,
    pub kind: NodeKind,
    pub object_type: String,
    pub si_storable: SiStorable,
}

impl Node {
    #[tracing::instrument(level = "trace")]
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: Option<String>,
        kind: NodeKind,
        object_type: impl Into<String> + std::fmt::Debug,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: String,
    ) -> NodeResult<Node> {
        let name = crate::models::generate_name(name);
        let object_type = object_type.into();
        let si_storable = SiStorable::new(
            db,
            "node",
            billing_account_id.clone(),
            organization_id.clone(),
            workspace_id.clone(),
            Some(created_by_user_id.clone()),
        )
        .await?;

        let id = si_storable.object_id.clone();

        let node = Node {
            id: id.clone(),
            positions: HashMap::new(),
            kind,
            object_type: object_type.clone(),
            si_storable,
        };
        insert_model(db, nats, &node.id, &node).await?;

        match node.kind {
            NodeKind::Entity => {
                Entity::new(
                    db,
                    nats,
                    Some(name),
                    None,
                    id,
                    object_type,
                    true,
                    billing_account_id,
                    organization_id,
                    workspace_id,
                    change_set_id,
                    edit_session_id,
                    created_by_user_id,
                )
                .await?;
            }
            NodeKind::System => {
                System::new(
                    db,
                    nats,
                    Some(name),
                    None,
                    id,
                    false,
                    billing_account_id,
                    organization_id,
                    workspace_id,
                    change_set_id,
                    edit_session_id,
                    created_by_user_id,
                )
                .await?;
            }
        }

        Ok(node)
    }

    pub async fn include_in_system(
        &self,
        db: &Db,
        nats: &Connection,
        system_id: impl Into<String>,
    ) -> NodeResult<Edge> {
        let system_id = system_id.into();
        let system = System::get_any(db, &system_id).await?;
        let object_id = self.get_object_id(db).await?;
        let edge = Edge::new(
            db,
            nats,
            Vertex::new(
                &system.node_id,
                &system.id,
                "output",
                &system.si_storable.type_name,
            ),
            Vertex::new(&self.id, object_id, "input", &self.object_type),
            false,
            EdgeKind::Includes,
            self.si_storable.billing_account_id.clone(),
            self.si_storable.organization_id.clone(),
            self.si_storable.workspace_id.clone(),
            None,
        )
        .await?;
        Ok(edge)
    }

    pub async fn get_object_id(&self, db: &Db) -> NodeResult<String> {
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"{type_name}\"
            AND a.nodeId = $node_id
          LIMIT 1
        ",
            bucket = db.bucket_name,
            type_name = self.kind,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("node_id".into(), serde_json::json![&self.id]);
        let mut query_results: Vec<serde_json::Value> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(NodeError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            let object_id = String::from(result["id"].as_str().ok_or(NodeError::NoObjectId)?);
            Ok(object_id)
        }
    }

    pub async fn get_head_object<T: DeserializeOwned + std::fmt::Debug>(
        &self,
        db: &Db,
    ) -> NodeResult<T> {
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"{type_name}\"
            AND a.nodeId = $node_id
            AND a.head = true
          LIMIT 1
        ",
            bucket = db.bucket_name,
            type_name = self.kind,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("node_id".into(), serde_json::json![&self.id]);
        let mut query_results: Vec<T> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(NodeError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get_object_projection<T: DeserializeOwned + std::fmt::Debug>(
        &self,
        db: &Db,
        change_set_id: impl AsRef<str>,
    ) -> NodeResult<T> {
        let change_set_id = change_set_id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"{type_name}\"
            AND a.siChangeSet.changeSetId = $change_set_id
            AND a.nodeId = $node_id
            AND a.head = false
          LIMIT 1
        ",
            bucket = db.bucket_name,
            type_name = self.kind,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("node_id".into(), serde_json::json![&self.id]);
        named_params.insert("change_set_id".into(), serde_json::json![change_set_id]);
        let mut query_results: Vec<T> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(NodeError::NoProjection)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get(
        db: &Db,
        node_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> NodeResult<Node> {
        let node_id = node_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();

        let node: Node = get_model(db, node_id, billing_account_id).await?;
        Ok(node)
    }
}
