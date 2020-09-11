use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::Db;
use crate::models::{
    insert_model, Entity, EntityError, ModelError, SiChangeSetError, SiStorable, SiStorableError,
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
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type NodeResult<T> = Result<T, NodeError>;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateRequest {
    pub name: Option<String>,
    pub kind: NodeKind,
    pub object_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateReply {
    pub item: Node,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Position {
    x: u64,
    y: u64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Entity,
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
            created_by_user_id.clone(),
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

        let _entity = Entity::new(
            db,
            Some(name),
            None,
            id,
            object_type,
            billing_account_id,
            organization_id,
            workspace_id,
            change_set_id,
            edit_session_id,
            created_by_user_id,
        )
        .await?;

        Ok(node)
    }
}
