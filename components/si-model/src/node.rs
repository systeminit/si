use crate::{Entity, EntityError, SiStorable, Veritech};
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("entity creation error: {0}")]
    Entity(#[from] EntityError),
    #[error("entity nodes require at least one system")]
    EntityRequiresSystem,
    #[error("no head object found; logic error")]
    NoHead,
    #[error("no object id; bug!")]
    NoObjectId,
    #[error("no projection object found")]
    NoProjection,
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type NodeResult<T> = Result<T, NodeError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub object_type: String,
    pub object_id: String,
    pub si_storable: SiStorable,
}

impl Node {
    pub async fn new(
        pg: &PgPool,
        txn: &PgTxn<'_>,
        nats_conn: &NatsConn,
        nats: &NatsTxn,
        veritech: &Veritech,
        name: Option<String>,
        object_type: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> NodeResult<Node> {
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let object_type = object_type.as_ref();

        let row = txn
            .query_one(
                "SELECT object FROM node_create_v1($1, $2)",
                &[&object_type, &workspace_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let mut node: Node = serde_json::from_value(json)?;
        let entity = Entity::new(
            &pg,
            &txn,
            &nats_conn,
            &nats,
            &veritech,
            name,
            None,
            &node.id,
            object_type,
            workspace_id,
            change_set_id,
            edit_session_id,
        )
        .await?;
        node.update_object_id(&txn, &entity.id).await?;

        //let _event = Event::node_entity_create(&pg, &nats_conn, &node, &entity, None).await?;
        let json: serde_json::Value = serde_json::to_value(&node)?;
        nats.publish(&json).await?;

        Ok(node)
    }

    async fn update_object_id(
        &mut self,
        txn: &PgTxn<'_>,
        object_id: impl AsRef<str>,
    ) -> NodeResult<()> {
        let object_id = object_id.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM node_update_object_id_v1($1, $2)",
                &[&self.id, &object_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let node: Node = serde_json::from_value(json)?;
        *self = node;
        Ok(())
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> NodeResult<()> {
        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM node_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: Node = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn get(txn: &PgTxn<'_>, node_id: impl AsRef<str>) -> NodeResult<Node> {
        let id = node_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM node_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_for_object_id(
        txn: &PgTxn<'_>,
        object_id: impl AsRef<str>,
        change_set_id: Option<&String>,
    ) -> NodeResult<Node> {
        let object_id = object_id.as_ref();
        let entity = Entity::for_head_or_change_set(&txn, &object_id, change_set_id).await?;
        let node = Node::get(&txn, entity.node_id).await?;
        Ok(node)
    }
}
