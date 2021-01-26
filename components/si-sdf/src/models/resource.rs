use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use strum_macros::Display;
use thiserror::Error;
use tracing::{error, warn};

use crate::data::{NatsConn, NatsTxnError, PgPool, PgTxn};
use crate::models::{
    next_update_clock, Edge, EdgeError, EdgeKind, Entity, Event, EventError, Node, SiStorable,
    UpdateClockError,
};
use crate::veritech::Veritech;

const RESOURCE_GET_ANY_BY_ENTITY_ID: &str =
    include_str!("../data/queries/resource_get_any_by_entity_id.sql");
const RESOURCE_GET_HEAD_BY_ENTITY_ID: &str =
    include_str!("../data/queries/resource_get_head_by_entity_id.sql");
const RESOURCE_GET_ANY_BY_NODE_ID: &str =
    include_str!("../data/queries/resource_get_any_by_node_id.sql");
const RESOURCE_GET_HEAD_BY_NODE_ID: &str =
    include_str!("../data/queries/resource_get_head_by_node_id.sql");

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("no resource found: {0} {1}")]
    NoResource(String, String),
    #[error("missing change set id on resource projection save")]
    MissingChangeSetId,
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("entity error: {0}")]
    Entity(String),
    #[error("node error: {0}")]
    Node(String),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("event error: {0}")]
    Event(#[from] EventError),
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncPredecessor {
    pub entity: Entity,
    pub resource: Resource,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceRequest<'a> {
    pub system_id: &'a str,
    pub node: &'a Node,
    pub entity: &'a Entity,
    pub resource: &'a Resource,
    pub predecessors: Vec<VeritechSyncPredecessor>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceUpdate {
    pub state: serde_json::Value,
    pub status: ResourceStatus,
    pub health: ResourceHealth,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VeritechSyncResourceReply {
    pub resource: VeritechSyncResourceUpdate,
}

pub type ResourceResult<T> = Result<T, ResourceError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResourceStatus {
    Pending,
    InProgress,
    Created,
    Failed,
    Deleted,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResourceHealth {
    Ok,
    Warning,
    Error,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub state: serde_json::Value,
    pub status: ResourceStatus,
    pub health: ResourceHealth,
    pub system_id: String,
    pub node_id: String,
    pub entity_id: String,
    pub change_set_id: Option<String>,
    pub si_storable: SiStorable,
}

impl Resource {
    pub async fn new(
        pg: &PgPool,
        nats_conn: &NatsConn,
        state: serde_json::Value,
        system_id: impl AsRef<str>,
        node_id: impl AsRef<str>,
        entity_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let system_id = system_id.as_ref();
        let node_id = node_id.as_ref();
        let entity_id = entity_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let workspace_update_clock = next_update_clock(workspace_id).await?;

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let row = txn
            .query_one(
                "SELECT object FROM resource_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    &state,
                    &ResourceStatus::Pending.to_string(),
                    &ResourceHealth::Unknown.to_string(),
                    &timestamp,
                    &unix_timestamp,
                    &system_id,
                    &node_id,
                    &entity_id,
                    &change_set_id,
                    &workspace_id,
                    &workspace_update_clock.epoch,
                    &workspace_update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;

        txn.commit().await?;
        nats.commit().await?;

        let object: Resource = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_any_by_entity_id(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();
        let change_set_id = change_set_id.as_ref();

        let row = txn
            .query_one(
                RESOURCE_GET_ANY_BY_ENTITY_ID,
                &[&entity_id, &system_id, &change_set_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Resource = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_head_by_entity_id(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();

        let row = txn
            .query_one(RESOURCE_GET_HEAD_BY_ENTITY_ID, &[&entity_id, &system_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Resource = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_any_by_node_id(
        txn: &PgTxn<'_>,
        node_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let node_id = node_id.as_ref();
        let system_id = system_id.as_ref();
        let change_set_id = change_set_id.as_ref();

        let row = txn
            .query_one(
                RESOURCE_GET_ANY_BY_NODE_ID,
                &[&node_id, &system_id, &change_set_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Resource = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_head_by_node_id(
        txn: &PgTxn<'_>,
        node_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let node_id = node_id.as_ref();
        let system_id = system_id.as_ref();

        let row = txn
            .query_one(RESOURCE_GET_HEAD_BY_NODE_ID, &[&node_id, &system_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Resource = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn from_update_for_self(
        &mut self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        state: serde_json::Value,
        status: ResourceStatus,
        health: ResourceHealth,
        change_set_id: Option<String>,
    ) -> ResourceResult<()> {
        self.state = state;
        self.status = status;
        self.health = health;
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        self.unix_timestamp = unix_timestamp;
        self.timestamp = timestamp;

        if change_set_id.is_some() {
            self.save_projection(pg, nats_conn).await?;
        } else {
            self.save_head(pg, nats_conn).await?;
        }
        Ok(())
    }

    pub async fn from_update(
        pg: &PgPool,
        nats_conn: &NatsConn,
        state: serde_json::Value,
        status: ResourceStatus,
        health: ResourceHealth,
        hypothetical: bool,
        system_id: impl AsRef<str>,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();
        let change_set_id = change_set_id.as_ref();

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;

        let mut resource =
            Resource::get_any_by_entity_id(&txn, entity_id, &system_id, &change_set_id).await?;
        resource.state = state;
        resource.status = status;
        resource.health = health;
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        resource.unix_timestamp = unix_timestamp;
        resource.timestamp = timestamp;

        if hypothetical {
            resource.change_set_id = Some(String::from(change_set_id));
            resource.save_projection(&pg, &nats_conn).await?;
        } else {
            resource.save_head(&pg, &nats_conn).await?;
        }
        Ok(resource)
    }

    pub async fn save_head(&mut self, pg: &PgPool, nats_conn: &NatsConn) -> ResourceResult<()> {
        let update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = update_clock;

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM resource_save_head_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;

        txn.commit().await?;
        nats.commit().await?;

        let mut updated: Resource = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn save_projection(
        &mut self,
        pg: &PgPool,
        nats_conn: &NatsConn,
    ) -> ResourceResult<()> {
        if self.change_set_id.is_none() {
            return Err(ResourceError::MissingChangeSetId);
        }

        let workspace_update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = workspace_update_clock;

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one(
                "SELECT object FROM resource_save_projection_v1($1)",
                &[&json],
            )
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;

        txn.commit().await?;
        nats.commit().await?;

        // Note: Fletcher and Adam think this is a good idea right now, but saving to head on every
        // projection save may end up being a bug at some point. Hope you have a great day!
        self.save_head(pg, nats_conn).await?;

        let mut updated: Resource = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn sync(
        &self,
        pg: PgPool,
        txn: &PgTxn<'_>,
        nats_conn: NatsConn,
        veritech: Veritech,
    ) -> ResourceResult<()> {
        let entity: Entity = if let Some(ref change_set_id) = self.change_set_id {
            Entity::get_projection_or_head(txn, &self.entity_id, change_set_id)
                .await
                .map_err(|e| ResourceError::Entity(e.to_string()))?
        } else {
            Entity::get_head(txn, &self.entity_id)
                .await
                .map_err(|e| ResourceError::Entity(e.to_string()))?
        };

        let event = Event::sync_resource(&pg, &nats_conn, &entity, &self.system_id, None).await?;

        let node = Node::get(txn, &self.node_id)
            .await
            .map_err(|e| ResourceError::Node(e.to_string()))?;

        let myself = self.clone();

        let predecessors = self.veritech_sync_predecessors(txn).await?;

        tokio::spawn(async move {
            call_veritech_sync_resource(
                pg,
                nats_conn,
                veritech,
                event,
                node,
                entity,
                myself,
                predecessors,
            )
            .await
        });
        Ok(())
    }

    async fn veritech_sync_predecessors(
        &self,
        txn: &PgTxn<'_>,
    ) -> ResourceResult<Vec<VeritechSyncPredecessor>> {
        let predecessor_edges =
            Edge::direct_predecessor_edges_by_node_id(txn, &EdgeKind::Configures, &self.id).await?;

        let mut predecessors = Vec::new();

        for edge in predecessor_edges {
            let (mut edge_entity, edge_resource) = if let Some(ref change_set_id) =
                self.change_set_id
            {
                let edge_entity =
                    Entity::get_projection_or_head(txn, &edge.tail_vertex.object_id, change_set_id)
                        .await
                        .map_err(|e| ResourceError::Entity(e.to_string()))?;
                let edge_resource = Resource::get_any_by_entity_id(
                    txn,
                    &edge_entity.id,
                    &self.system_id,
                    change_set_id,
                )
                .await?;
                (edge_entity, edge_resource)
            } else {
                let edge_entity = Entity::get_head(txn, &edge.tail_vertex.object_id)
                    .await
                    .map_err(|e| ResourceError::Entity(e.to_string()))?;
                let edge_resource =
                    Resource::get_head_by_entity_id(txn, &edge_entity.id, &self.system_id).await?;
                (edge_entity, edge_resource)
            };
            edge_entity
                .update_properties_if_secret(txn)
                .await
                .map_err(|e| ResourceError::Entity(e.to_string()))?;
            let predecessor = VeritechSyncPredecessor {
                entity: edge_entity,
                resource: edge_resource,
            };
            predecessors.push(predecessor);
        }

        Ok(predecessors)
    }
}

async fn call_veritech_sync_resource(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    mut event: Event,
    this_node: Node,
    entity: Entity,
    mut resource: Resource,
    predecessors: Vec<VeritechSyncPredecessor>,
) {
    let mut conn = match pg.pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!(?e, "cannot get connection to sync resource");
            return;
        }
    };
    let this_txn = match conn.transaction().await {
        Ok(txn) => txn,
        Err(e) => {
            error!(?e, "cannot get transaction to sync resource");
            return;
        }
    };
    let this_nats = nats_conn.transaction();

    let request = VeritechSyncResourceRequest {
        system_id: &resource.system_id,
        resource: &resource,
        node: &this_node,
        entity: &entity,
        predecessors,
    };

    let sync_reply: VeritechSyncResourceReply = match veritech
        .send(&pg, &nats_conn, "/ws/syncResource", request, &event)
        .await
    {
        Ok(Some(sync_reply)) => sync_reply,
        Ok(None) => {
            warn!("vertich sync got an okay None");
            if let Err(e) = event.unknown(&pg, &nats_conn).await {
                warn!(?e, "cannot write event unknown to db");
            }
            return;
        }
        Err(err) => {
            warn!("veritech got an error from the sync request; err={:?}", err);
            if let Err(e) = event.unknown(&pg, &nats_conn).await {
                warn!(?e, "cannot write event unknown to db");
            }
            return;
        }
    };
    if sync_reply.resource.status == ResourceStatus::Failed {
        warn!("veritech sync reply reports status is: resource failed");
        if let Err(e) = event.error(&pg, &nats_conn).await {
            warn!(?e, "cannot write event error to db");
        }
    } else {
        if let Err(e) = event.success(&pg, &nats_conn).await {
            warn!(?e, "cannot write event success to db");
        }
    }
    if let Err(e) = resource
        .from_update_for_self(
            &pg,
            &nats_conn,
            sync_reply.resource.state,
            sync_reply.resource.status,
            sync_reply.resource.health,
            resource.change_set_id.clone(),
        )
        .await
    {
        warn!("cannot update resource from response: {}", e);
        return;
    }
    if let Err(e) = this_nats.commit().await {
        error!(?e, "cannot commit nats transaction for sync resource");
    }
    if let Err(e) = this_txn.commit().await {
        error!(?e, "cannot commit transaction for sync resource");
    }
}
