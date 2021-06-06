use std::collections::HashMap;

use chrono::Utc;
use futures::{future::BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json;
use si_data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};
use strum_macros::Display;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::{
    workflow::selector::{SelectionEntry, SelectionEntryPredecessor},
    Edge, EdgeError, EdgeKind, Entity, Node, SiStorable, Veritech, VeritechError,
};

const RESOURCE_GET_BY_ENTITY_AND_SYSTEM: &str =
    include_str!("./queries/resource_get_by_entity_and_system.sql");
const RESOURCES_FOR_ENTITY: &str = include_str!("./queries/resource_for_entity.sql");

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
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("entity error: {0}")]
    Entity(String),
    #[error("node error: {0}")]
    Node(String),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("veritech error: {0}")]
    Veritech(#[from] VeritechError),
    #[error("oneshot recv error: {0}")]
    Recv(#[from] oneshot::error::RecvError),
    #[error("workflow error: {0}")]
    Workflow(String),
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
    pub status: ResourceInternalStatus,
    pub health: ResourceInternalHealth,
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
pub enum ResourceInternalStatus {
    Pending,
    InProgress,
    Created,
    Failed,
    Deleted,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResourceInternalHealth {
    Ok,
    Warning,
    Error,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Predecessor {
    entity: Entity,
    resource: Option<Resource>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncRequest<'a> {
    entity: &'a Entity,
    resource: &'a Resource,
    system: &'a Entity,
    context: Vec<SelectionEntryPredecessor>,
    resource_context: Vec<Resource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncFinish {
    pub data: serde_json::Value,
    pub state: String,
    pub health: String,
    pub internal_status: ResourceInternalStatus,
    pub internal_health: ResourceInternalHealth,
    pub sub_resources: HashMap<String, SubResource>,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SyncProtocol {
    Start(bool),
    Finish(SyncFinish),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubResource {
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub internal_status: ResourceInternalStatus,
    pub internal_health: ResourceInternalHealth,
    pub state: String,
    pub health: String,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub internal_status: ResourceInternalStatus,
    pub internal_health: ResourceInternalHealth,
    pub state: String,
    pub health: String,
    pub data: serde_json::Value,
    pub sub_resources: HashMap<String, SubResource>,
    pub error: Option<String>,
    pub system_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub si_storable: SiStorable,
}

impl Resource {
    // NOTE: new takes a PgPool and NatsConn intentionally--a resource must be immediately
    // available for the entire system when created and not batched into another transaction. In
    // this way we prevent multiple Resource representations being created and other concurrent
    // tasks can load this representations immediately. You're welcome!
    pub async fn new(
        pg: &PgPool,
        nats_conn: &NatsConn,
        state: serde_json::Value,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> ResourceResult<Resource> {
        let system_id = system_id.as_ref();
        let entity_id = entity_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let row = txn
            .query_one(
                "SELECT object FROM resource_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &state,
                    &ResourceInternalStatus::Pending.to_string(),
                    &ResourceInternalHealth::Unknown.to_string(),
                    &timestamp,
                    &unix_timestamp,
                    &system_id,
                    &entity_id,
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;

        nats.publish(&json).await?;

        let object: Resource = serde_json::from_value(json)?;

        txn.commit().await?;
        nats.commit().await?;

        Ok(object)
    }

    pub async fn get_by_entity_and_system(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
    ) -> ResourceResult<Option<Self>> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();
        let row = match txn
            .query_opt(RESOURCE_GET_BY_ENTITY_AND_SYSTEM, &[&entity_id, &system_id])
            .await?
        {
            Some(row) => row,
            None => return Ok(None),
        };
        let object: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(object)?;

        Ok(Some(object))
    }

    pub async fn for_entity(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
    ) -> ResourceResult<Vec<Self>> {
        let mut results = vec![];
        let entity_id = entity_id.as_ref();
        let rows = txn.query(RESOURCES_FOR_ENTITY, &[&entity_id]).await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let object: Resource = serde_json::from_value(json)?;
            results.push(object);
        }
        Ok(results)
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> ResourceResult<()> {
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        self.timestamp = timestamp;
        self.unix_timestamp = unix_timestamp;

        let json = serde_json::to_value(&self)?;

        let row = txn
            .query_one("SELECT object FROM resource_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;

        let mut updated: Self = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn sync(
        self,
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
    ) -> ResourceResult<()> {
        self.sync_inner(pg, nats_conn, veritech, None).await
    }

    pub async fn await_sync(
        &mut self,
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
    ) -> ResourceResult<()> {
        let (tx, rx) = oneshot::channel();
        self.sync_inner(pg, nats_conn, veritech, Some(tx)).await?;

        let mut updated = rx.await??;
        std::mem::swap(self, &mut updated);

        Ok(())
    }

    async fn sync_inner(
        &self,
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
        wait_channel: Option<oneshot::Sender<ResourceResult<Self>>>,
    ) -> ResourceResult<()> {
        let resource = self.clone();
        let resource_id = self.id.clone();

        tokio::spawn(async move {
            let result = resource.sync_task(pg, nats_conn, veritech).await;

            if let Err(ref err) = result {
                dbg!("syncing resource {} failed {:?}", resource_id, err);
            }
            if let Some(wait_channel) = wait_channel {
                let _ = wait_channel.send(result);
            }
        });
        Ok(())
    }

    pub async fn sync_task(
        mut self,
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
    ) -> ResourceResult<Self> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;

        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<SyncProtocol>();

        let entity = Entity::for_head(&txn, &self.entity_id)
            .await
            .map_err(|err| ResourceError::Entity(err.to_string()))?;
        let system = Entity::for_head(&txn, &self.system_id)
            .await
            .map_err(|err| ResourceError::Entity(err.to_string()))?;

        let context = SelectionEntry::new(&txn, &entity.id, &system)
            .await
            .map_err(|e| ResourceError::Workflow(e.to_string()))?;

        let predecessors_for_resources =
            Edge::all_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &entity.id)
                .await?;
        let mut resource_context: Vec<Resource> = vec![];
        for edge in predecessors_for_resources.iter() {
            let resource =
                Resource::get_by_entity_and_system(&txn, &edge.tail_vertex.object_id, &system.id)
                    .await?;
            match resource {
                Some(r) => resource_context.push(r),
                None => continue,
            }
        }

        txn.commit().await?;

        let request = SyncRequest {
            entity: &entity,
            resource: &self,
            system: &system,
            context: context.context,
            resource_context,
        };

        veritech
            .send_async("syncResource", request, progress_tx)
            .await?;

        while let Some(message) = progress_rx.recv().await {
            match message {
                SyncProtocol::Start(_) => {}
                SyncProtocol::Finish(finish) => {
                    let txn = conn.transaction().await?;
                    let nats = nats_conn.transaction();

                    let current_time = Utc::now();
                    let unix_timestamp = current_time.timestamp_millis();
                    let timestamp = format!("{}", current_time);

                    self.state = finish.state;
                    self.health = finish.health;
                    self.data = finish.data;
                    self.internal_status = finish.internal_status;
                    self.internal_health = finish.internal_health;
                    self.sub_resources = finish.sub_resources;
                    self.unix_timestamp = unix_timestamp;
                    self.timestamp = timestamp;
                    self.error = finish.error.clone();

                    if let Some(err_msg) = finish.error {
                        dbg!("uh oh, error when syncing resource: {}", err_msg);
                    }

                    self.save(&txn, &nats).await?;

                    txn.commit().await?;
                    nats.commit().await?;
                }
            }
        }

        Ok(self)
    }
}

pub fn sync_resource(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
    entity: &Entity,
) -> BoxFuture<'static, ResourceResult<()>> {
    let entity = entity.clone();
    let pg = pg.clone();
    let veritech = veritech.clone();
    let nats_conn = nats_conn.clone();
    let r = async move {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let systems: Vec<Entity> = Entity::get_head_by_name_and_entity_type(
            &txn,
            "production",
            "system",
            &entity.si_storable.workspace_id,
        )
        .await
        .map_err(|e| ResourceError::Entity(e.to_string()))?
        .into_iter()
        .filter(|s| s.si_storable.workspace_id == entity.si_storable.workspace_id)
        .collect();
        let system_id = systems.first().unwrap().id.clone();
        let mut r = match Resource::get_by_entity_and_system(&txn, &entity.id, &system_id).await? {
            Some(r) => r,
            None => {
                Resource::new(
                    &pg,
                    &nats_conn,
                    serde_json::json!([]),
                    &entity.id,
                    &system_id,
                    &entity.si_storable.workspace_id,
                )
                .await?
            }
        };
        r.await_sync(pg.clone(), nats_conn.clone(), veritech.clone())
            .await?;
        txn.commit().await?;
        Ok(())
    };
    r.boxed()
}
