use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

use crate::data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};
use crate::veritech::Veritech;

use crate::models::{
    list_model, next_update_clock, Edge, EdgeError, EdgeKind, Entity, Event, EventError, ListReply,
    ModelError, OrderByDirection, PageToken, Query, Resource, ResourceError, SiStorable, System,
    SystemError, UpdateClockError, Vertex,
};

use crate::models::{OpReply, OpRequest};

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("error with linked entity: {0}")]
    Entity(String),
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
    #[error("entity nodes require at least one system")]
    EntityRequiresSystem,
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("event error: {0}")]
    Event(#[from] EventError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
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
    pub system_ids: Option<Vec<String>>,
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
pub struct PatchConfiguredByRequest {
    pub node_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchConfiguredByReply {
    pub edge: Edge,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchSetPositionRequest {
    pub context: String,
    // pub position: Position,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchSetPositionReply {
    pub context: String,
    // pub position: Position,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceRequest {
    pub system_id: String,
    pub change_set_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceReply {
    //pub resource: Resource,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchOp {
    IncludeSystem(PatchIncludeSystemRequest),
    ConfiguredBy(PatchConfiguredByRequest),
    SyncResource(SyncResourceRequest),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchReply {
    IncludeSystem(PatchIncludeSystemReply),
    ConfiguredBy(PatchConfiguredByReply),
    SetPosition(PatchSetPositionReply),
    SyncResource(SyncResourceReply),
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
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
        kind: NodeKind,
        object_type: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
        system_ids: Option<Vec<String>>,
    ) -> NodeResult<Node> {
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let object_type = object_type.as_ref();

        let workspace_update_clock = next_update_clock(workspace_id).await?;

        let row = txn
            .query_one(
                "SELECT object FROM node_create_v1($1, $2, $3, $4, $5)",
                &[
                    &kind.to_string(),
                    &object_type,
                    &workspace_id,
                    &workspace_update_clock.epoch,
                    &workspace_update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let mut node: Node = serde_json::from_value(json)?;

        match node.kind {
            NodeKind::Entity => {
                let system_ids = system_ids.ok_or(NodeError::EntityRequiresSystem)?;
                let entity = Entity::new(
                    &pg,
                    &txn,
                    &nats_conn,
                    &nats,
                    name,
                    None,
                    &node.id,
                    object_type,
                    workspace_id,
                    change_set_id,
                    edit_session_id,
                    system_ids.clone(),
                )
                .await
                .map_err(|e| NodeError::Entity(e.to_string()))?;
                node.update_object_id(&txn, &entity.id).await?;
                let _event =
                    Event::node_entity_create(&pg, &nats_conn, &node, &entity, None).await?;
                for system_id in system_ids.iter() {
                    node.sync_resource(
                        pg,
                        txn,
                        nats_conn,
                        veritech,
                        system_id,
                        Some(String::from(change_set_id)),
                    )
                    .await?;
                }
            }
            NodeKind::System => {
                let system = System::new(
                    &txn,
                    &nats,
                    name,
                    None,
                    &node.id,
                    workspace_id,
                    change_set_id,
                    edit_session_id,
                )
                .await?;
                node.update_object_id(&txn, &system.id).await?;
            }
        }
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
        let workspace_update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = workspace_update_clock;

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

    pub async fn sync_resource(
        &self,
        pg: &PgPool,
        txn: &PgTxn<'_>,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        system_id: impl AsRef<str>,
        change_set_id: Option<String>,
    ) -> NodeResult<()> {
        let system_id = system_id.as_ref();
        let resource: Resource = if let Some(ref change_set_id) = change_set_id {
            Resource::get_any_by_node_id(&txn, &self.id, system_id, change_set_id)
                .await
                .map_err(|e| ResourceError::Entity(e.to_string()))?
        } else {
            Resource::get_head_by_node_id(&txn, &self.id, system_id)
                .await
                .map_err(|e| ResourceError::Entity(e.to_string()))?
        };
        resource
            .sync(pg.clone(), txn, nats_conn.clone(), veritech.clone())
            .await?;
        Ok(())
    }

    pub async fn configured_by(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        node_id: impl Into<String>,
    ) -> NodeResult<Edge> {
        let node_id = node_id.into();
        let node = Node::get(&txn, &node_id).await?;
        let node_object_id = node.get_object_id();

        let object_id = self.get_object_id();

        let edge = Edge::new(
            txn,
            nats,
            Vertex::new(&node.id, &node_object_id, "output", &node.object_type),
            Vertex::new(&self.id, object_id, "input", &self.object_type),
            false,
            EdgeKind::Configures,
            &self.si_storable.workspace_id,
        )
        .await?;

        Ok(edge)
    }

    pub async fn include_in_system(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        system_id: impl Into<String>,
    ) -> NodeResult<Edge> {
        let system_id = system_id.into();
        let system = System::get_any(txn, &system_id).await?;

        let object_id = self.get_object_id();

        let edge = Edge::new(
            txn,
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
            &self.si_storable.workspace_id,
        )
        .await?;

        Ok(edge)
    }

    pub fn get_object_id(&self) -> String {
        return String::from(&self.object_id);
    }

    pub async fn get_head_object_entity(&self, txn: &PgTxn<'_>) -> NodeResult<Entity> {
        let e = Entity::get_head(&txn, &self.object_id)
            .await
            .map_err(|e| NodeError::Entity(e.to_string()))?;
        Ok(e)
    }

    pub async fn get_head_object_system(&self, txn: &PgTxn<'_>) -> NodeResult<System> {
        let s = System::get_head(&txn, &self.object_id).await?;
        Ok(s)
    }

    pub async fn get_projection_object_entity(
        &self,
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
    ) -> NodeResult<Entity> {
        let change_set_id = change_set_id.as_ref();
        let e = Entity::get_projection(&txn, &self.object_id, &change_set_id)
            .await
            .map_err(|e| NodeError::Entity(e.to_string()))?;
        Ok(e)
    }

    pub async fn get_projection_object_system(
        &self,
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
    ) -> NodeResult<System> {
        let change_set_id = change_set_id.as_ref();
        let s = System::get_projection(&txn, &self.object_id, &change_set_id)
            .await
            .map_err(|e| NodeError::Entity(e.to_string()))?;
        Ok(s)
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

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> NodeResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "nodes",
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
}
