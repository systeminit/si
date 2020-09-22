use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
use crate::models::{
    get_model, insert_model, Entity, EntityError, ModelError, OpReply, OpRequest, SiChangeSetError,
    SiStorable, SiStorableError, System, SystemError,
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
pub struct PatchRequest {
    pub op: OpRequest,
    pub organization_id: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchReply {
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub name: String,
    pub positions: HashMap<String, Position>,
    pub kind: NodeKind,
    pub object_type: String,
    pub si_storable: SiStorable,
}

impl Node {
    #[tracing::instrument(level = "trace")]
    pub async fn new(
        db: &Db,
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
            name: name.clone(),
            positions: HashMap::new(),
            kind,
            object_type: object_type.clone(),
            si_storable,
        };
        insert_model(db, &node.id, &node).await?;

        match node.kind {
            NodeKind::Entity => {
                Entity::new(
                    db,
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

    pub async fn get_head_object<T: DeserializeOwned + std::fmt::Debug>(
        &self,
        db: &Db,
    ) -> NodeResult<T> {
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"entity\"
            AND a.nodeId = $node_id
            AND a.head = true
          LIMIT 1
        ",
            bucket = db.bucket_name
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
          WHERE a.siStorable.typeName = \"entity\"
            AND a.siChangeSet.changeSetId = $change_set_id
            AND a.nodeId = $node_id
            AND a.head = false
          LIMIT 1
        ",
            bucket = db.bucket_name
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
