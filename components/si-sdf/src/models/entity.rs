use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{error, info, trace};

use crate::data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};

use crate::models::{
    list_model, next_update_clock, Edge, EdgeError, EdgeKind, EncryptedSecret, ListReply,
    ModelError, OrderByDirection, PageToken, Query, Resource, ResourceError, SecretError,
    SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiStorable, System, SystemError,
    UpdateClockError, Vertex,
};

const ENTITY_GET_ANY: &str = include_str!("../data/queries/entity_get_any.sql");
const ENTITY_GET_HEAD: &str = include_str!("../data/queries/entity_get_head.sql");
const ENTITY_GET_PROJECTION: &str = include_str!("../data/queries/entity_get_projection.sql");
const ENTITY_GET_ALL: &str = include_str!("../data/queries/entity_get_all.sql");
const ENTITY_GET_HEAD_OR_BASE: &str = include_str!("../data/queries/entity_get_head_or_base.sql");

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("si_change_set error: {0}")]
    SiChangeSet(#[from] SiChangeSetError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("error making http call: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("no head entity found; logic error")]
    NoHead,
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("no override system found: {0}")]
    Override(String),
    #[error("invalid entity; missing object type")]
    MissingObjectType,
    #[error("invalid entity; missing node id")]
    MissingId,
    #[error("missing field: {0}")]
    Missing(String),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("node error: {0}")]
    Node(String),
    #[error("system error: {0}")]
    System(#[from] SystemError),
    #[error("no systems were provided; must have at least 1!")]
    NotEnoughSystems,
    #[error("not found")]
    NotFound,
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("malformed database entry")]
    MalformedDatabaseEntry,
    #[error("missing change set in a save where it is required")]
    MissingChangeSet,
}

pub type EntityResult<T> = Result<T, EntityError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetReply {
    pub items: Vec<Entity>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: Entity,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculatePropertiesPredecessor {
    pub entity: Entity,
    pub resources: Vec<Resource>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculatePropertiesRequest<'a> {
    object_type: &'a str,
    entity: &'a serde_json::Value,
    predecessors: Vec<CalculatePropertiesPredecessor>,
    resources: Vec<Resource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculatePropertiesResponse {
    entity: serde_json::Value,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculateConfiguresRequest<'a> {
    entity: &'a serde_json::Value,
    configures: &'a serde_json::Value,
    systems: &'a serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculateConfiguresResponseCreateEntry {
    object_type: String,
    name: Option<String>,
    systems: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculateConfiguresResponseKeepEntry {
    id: String,
    systems: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculateConfiguresResponse {
    keep: Option<Vec<String>>,
    create: Option<Vec<CalculateConfiguresResponseCreateEntry>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntityProperties(HashMap<String, serde_json::Value>);

impl EntityProperties {
    pub fn new() -> Self {
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();
        map.insert("__baseline".to_string(), serde_json::json![{}]);
        EntityProperties(map)
    }

    pub fn get(&self, k: impl AsRef<str>) -> Option<&serde_json::Value> {
        let k = k.as_ref();
        self.0.get(k)
    }

    pub fn get_or_create_mut(&mut self, k: impl AsRef<str>) -> &mut serde_json::Value {
        let k = k.as_ref();
        if !self.0.contains_key(k) {
            self.0.insert(String::from(k), serde_json::json![{}]);
        }
        // Safe! We check right above.
        self.0.get_mut(k).unwrap()
    }

    pub fn get_property(
        &self,
        pointer: impl AsRef<str>,
        override_system: Option<&str>,
    ) -> EntityResult<Option<&serde_json::Value>> {
        let pointer = pointer.as_ref();
        let override_system = match override_system {
            Some(override_system) => override_system,
            None => "__baseline",
        };
        let properties = self
            .get(override_system)
            .ok_or(EntityError::Override(override_system.into()))?;
        Ok(properties.pointer(pointer))
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub object_type: String,
    pub description: String,
    pub expression_properties: EntityProperties,
    pub manual_properties: EntityProperties,
    pub inferred_properties: EntityProperties,
    pub properties: EntityProperties,
    pub node_id: String,
    pub head: bool,
    pub base: bool,
    pub si_storable: SiStorable,
    pub si_change_set: Option<SiChangeSet>,
}

impl Entity {
    pub async fn new(
        pg: &PgPool,
        txn: &PgTxn<'_>,
        nats_conn: &NatsConn,
        nats: &NatsTxn,
        name: Option<String>,
        description: Option<String>,
        node_id: impl AsRef<str>,
        object_type: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
        system_ids: Vec<String>,
    ) -> EntityResult<Entity> {
        let workspace_id = workspace_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let object_type = object_type.as_ref();
        let node_id = node_id.as_ref();

        if system_ids.len() == 0 {
            return Err(EntityError::NotEnoughSystems);
        }
        let name = crate::models::generate_name(name);
        let description = if description.is_some() {
            description.unwrap()
        } else {
            name.clone()
        };

        let workspace_update_clock = next_update_clock(workspace_id).await?;
        let change_set_update_clock = next_update_clock(change_set_id).await?;

        let row = txn
            .query_one(
                "SELECT object FROM entity_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    &name,
                    &description,
                    &object_type,
                    &node_id,
                    &change_set_id,
                    &edit_session_id,
                    &SiChangeSetEvent::Create.to_string(),
                    &workspace_id,
                    &workspace_update_clock.epoch,
                    &workspace_update_clock.update_count,
                    &change_set_update_clock.epoch,
                    &change_set_update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let mut entity: Entity = serde_json::from_value(json)?;
        match entity.si_change_set.as_ref() {
            Some(si_change_set) => {
                si_change_set
                    .create_change_set_participants(&txn, &nats, &entity.id, &workspace_id)
                    .await?;
            }
            None => return Err(EntityError::MalformedDatabaseEntry),
        }

        for system_id in system_ids {
            trace!(?system_id, ?entity, "getting system edge");
            let system = System::get_any(&txn, &system_id).await?;
            trace!(?system_id, ?system, ?entity, "adding sytem edge");
            Edge::new(
                &txn,
                &nats,
                Vertex::new(&system.node_id, &system.id, "output", "system"),
                Vertex::new(&entity.node_id, &entity.id, "input", &entity.object_type),
                false,
                EdgeKind::Includes,
                &workspace_id,
            )
            .await?;
            Resource::new(
                &pg,
                &nats_conn,
                serde_json::json![{}],
                &system_id,
                &node_id,
                &entity.id,
                &workspace_id,
                &change_set_id,
            )
            .await?;
        }

        entity.calculate_properties(&txn).await?;

        entity.save_base_and_projection(&txn, &nats).await?;

        Ok(entity)
    }

    pub async fn save_projection(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EntityResult<()> {
        self.head = false;
        self.base = false;
        if self.si_change_set.is_none() {
            return Err(EntityError::MissingChangeSet);
        }

        let workspace_update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = workspace_update_clock;
        let change_set_update_clock =
            next_update_clock(&self.si_change_set.as_ref().unwrap().change_set_id).await?;
        self.si_change_set.as_mut().unwrap().order_clock = change_set_update_clock;

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM entity_save_projection_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: Entity = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn save_base(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EntityResult<()> {
        self.head = false;
        self.base = true;
        if self.si_change_set.is_none() {
            return Err(EntityError::MissingChangeSet);
        }

        let workspace_update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = workspace_update_clock;
        let change_set_update_clock =
            next_update_clock(&self.si_change_set.as_ref().unwrap().change_set_id).await?;
        self.si_change_set.as_mut().unwrap().order_clock = change_set_update_clock;

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM entity_save_base_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: Entity = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    async fn save_base_and_projection(
        &mut self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
    ) -> EntityResult<()> {
        self.save_projection(&txn, &nats).await?;
        self.save_base(&txn, &nats).await?;
        Ok(())
    }

    pub async fn save_head(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EntityResult<()> {
        self.head = true;
        self.base = false;
        self.si_change_set = None;

        let update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = update_clock;

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM entity_save_head_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: Entity = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn calculate_properties(&mut self, txn: &PgTxn<'_>) -> EntityResult<()> {
        let mut json = serde_json::json![self];
        calculate_properties(txn, &mut json, None).await?;
        let new_entity: Entity = serde_json::from_value(json)?;
        *self = new_entity;
        Ok(())
    }

    pub async fn update_properties_if_secret(&mut self, txn: &PgTxn<'_>) -> EntityResult<()> {
        if let Some(secret_id) = self
            .properties
            .get_property("/secretId", None)?
            .map(|s| s.as_str())
            .flatten()
        {
            let secret = EncryptedSecret::get(&txn, secret_id).await?;
            let decrypted = secret.decrypt(&txn).await?;
            self.properties
                .get_or_create_mut("__baseline")
                .as_object_mut()
                .expect("__baseline must be a map")
                .insert("decrypted".into(), decrypted.message);
            self.properties
                .get_or_create_mut("__baseline")
                .as_object_mut()
                .expect("__baseline must be a map")
                .remove("secretId");
            self.manual_properties
                .get_or_create_mut("__baseline")
                .as_object_mut()
                .expect("__baseline must be a map")
                .remove("secretId");
        }
        Ok(())
    }

    pub async fn get_any(txn: &PgTxn<'_>, id: impl AsRef<str>) -> EntityResult<Entity> {
        let id = id.as_ref();
        let row = txn.query_one(ENTITY_GET_ANY, &[&id]).await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Entity = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_head_or_base(
        txn: &PgTxn<'_>,
        id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        let id = id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let row = txn
            .query_one(ENTITY_GET_HEAD_OR_BASE, &[&id, &change_set_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Entity = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_head(txn: &PgTxn<'_>, id: impl AsRef<str>) -> EntityResult<Entity> {
        let id = id.as_ref();
        let row = txn.query_one(ENTITY_GET_HEAD, &[&id]).await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Entity = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_projection(
        txn: &PgTxn<'_>,
        id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        let id = id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let row = txn
            .query_one(ENTITY_GET_PROJECTION, &[&id, &change_set_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Entity = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_projection_or_head(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        match Self::get_projection(txn, &entity_id, change_set_id).await {
            Ok(entity) => Ok(entity),
            Err(_) => Self::get_head(txn, entity_id).await,
        }
    }

    pub async fn get_all(txn: &PgTxn<'_>, entity_id: impl AsRef<str>) -> EntityResult<Vec<Entity>> {
        let mut results = Vec::new();
        let id = entity_id.as_ref();
        let rows = txn.query(ENTITY_GET_ALL, &[&id]).await?;

        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let object: Entity = serde_json::from_value(json)?;
            results.push(object);
        }
        Ok(results)
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> EntityResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "entities_head",
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

    //pub fn calculate_configures(
    //    &self,
    //    db: Db,
    //    nats: Connection,
    //) -> Pin<Box<dyn Future<Output = EntityResult<()>> + Send>> {
    //    let entity_json = serde_json::json![self];
    //    Box::pin(async move {
    //        calculate_configures(db, nats, entity_json).await?;
    //        Ok(())
    //    })
    //}
}

pub async fn calculate_properties(
    txn: &PgTxn<'_>,
    json: &mut serde_json::Value,
    projections: Option<&HashMap<String, serde_json::Value>>,
) -> EntityResult<()> {
    info!(?json, "calculating properties");
    let entity: Entity = serde_json::from_value(json.clone())?;
    let optional_change_set_id = if entity.head {
        None
    } else {
        entity
            .si_change_set
            .as_ref()
            .map(|sic| sic.change_set_id.clone())
    };
    let system_edges = Edge::by_kind_and_head_object_id_and_tail_type_name(
        &txn,
        &EdgeKind::Includes,
        &entity.id,
        "system",
    )
    .await?;

    let mut resources = Vec::new();
    for system_edge in system_edges.iter() {
        let resource = if let Some(si_change_set) = entity.si_change_set.as_ref() {
            Resource::get_any_by_entity_id(
                &txn,
                &entity.id,
                &system_edge.tail_vertex.object_id,
                &si_change_set.change_set_id,
            )
            .await?
        } else {
            Resource::get_head_by_entity_id(&txn, &entity.id, &system_edge.tail_vertex.object_id)
                .await?
        };
        resources.push(resource);
    }

    let predecessor_edges =
        Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &entity.id)
            .await?;
    let mut predecessors: Vec<CalculatePropertiesPredecessor> = Vec::new();
    for edge in predecessor_edges {
        // OMG, I'm so sorry - still sorry!
        let edge_entity: Entity = {
            let mut pe: Option<Entity> = None;
            if let Some(projection_map) = projections {
                match projection_map.get(&edge.tail_vertex.object_id) {
                    Some(entity_json) => {
                        let real_entity: Entity = serde_json::from_value(entity_json.clone())?;
                        pe = Some(real_entity);
                    }
                    None => pe = None,
                }
            }
            if let Some(entity) = pe {
                entity
            } else if let Some(ref change_set_id) = optional_change_set_id {
                Entity::get_projection_or_head(&txn, &edge.tail_vertex.object_id, change_set_id)
                    .await?
            } else {
                Entity::get_head(&txn, &edge.tail_vertex.object_id).await?
            }
        };
        let mut edge_resources: Vec<Resource> = Vec::new();
        trace!(?system_edges, "calculating edge resources for system edges");
        for system_edge in system_edges.iter() {
            let edge_resource = if let Some(ref change_set_id) = optional_change_set_id {
                Resource::get_any_by_entity_id(
                    &txn,
                    &edge_entity.id,
                    &system_edge.tail_vertex.object_id,
                    change_set_id,
                )
                .await?
            } else {
                Resource::get_head_by_entity_id(
                    &txn,
                    &edge_entity.id,
                    &system_edge.tail_vertex.object_id,
                )
                .await?
            };
            edge_resources.push(edge_resource);
        }

        let predecessor = CalculatePropertiesPredecessor {
            entity: edge_entity,
            resources: edge_resources,
        };
        predecessors.push(predecessor);
    }

    let object_type = json["objectType"]
        .as_str()
        .ok_or(EntityError::MissingObjectType)?;

    //let reqwest: reqwest::Client = if cfg!(test) {
    let reqwest = reqwest::Client::new();
    //} else {
    //    REQWEST.clone()
    //};
    let res = reqwest
        .post("http://localhost:5157/calculateProperties")
        .json(&CalculatePropertiesRequest {
            object_type,
            entity: json,
            predecessors,
            resources,
        })
        .send()
        .await?;
    let entity_result: CalculatePropertiesResponse = res.json().await?;
    trace!(
        ?entity_result,
        "calculate properties response from changeset"
    );
    *json = entity_result.entity;
    Ok(())
}

// pub fn calculate_configures(
//     db: Db,
//     nats: Connection,
//     entity_json: serde_json::Value,
// ) -> Pin<Box<dyn Future<Output = EntityResult<()>> + Send>> {
//     Box::pin(async move {
//          let id = entity_json["id"].as_str().ok_or(EntityError::MissingId)?;
//          let object_type = entity_json["objectType"]
//              .as_str()
//              .ok_or(EntityError::MissingObjectType)?;
//          let node_id = entity_json["nodeId"]
//              .as_str()
//              .ok_or(EntityError::Missing("nodeId".into()))?;
//          let change_set_id = entity_json["siChangeSet"]["changeSetId"]
//              .as_str()
//              .unwrap_or("fakemcfakerton");
//
//          // Get the list of edges this entity configures
//          let configures_edges =
//              Edge::by_kind_and_tail_object_id(&db, EdgeKind::Configures, id).await?;
//          let mut configures = Vec::with_capacity(configures_edges.len());
//          for edge in configures_edges.iter() {
//              let object = get_base_object(&db, &edge.head_vertex.object_id, change_set_id).await?;
//              configures.push(object);
//          }
//
//          // Get the list of systems this entity participates in
//          let system_edges = Edge::by_kind_and_head_object_id_and_tail_type_name(
//              &db,
//              EdgeKind::Includes,
//              id,
//              "system",
//          )
//          .await?;
//          let mut systems: Vec<System> = Vec::with_capacity(system_edges.len());
//          for system_edge in system_edges.iter() {
//              let system = System::get_any(&db, &system_edge.tail_vertex.object_id).await?;
//              systems.push(system);
//          }
//          trace!(?systems, ?node_id, "making nodes with the list of systems");
//
//          let res = REQWEST
//              .post("http://localhost:5157/calculateConfigures")
//              .json(&CalculateConfiguresRequest {
//                  entity: &entity_json,
//                  configures: &serde_json::json![configures],
//                  systems: &serde_json::json![systems],
//              })
//              .send()
//              .await?;
//          let configures_result: CalculateConfiguresResponse = res.json().await?;
//
//          // If any edge is not in the keep list from the callback, then we remove its
//          // connection.
//          if let Some(keep) = configures_result.keep {
//              for edge in configures_edges.into_iter() {
//                  if !keep.contains(&edge.head_vertex.object_id) {
//                      edge.delete(&db, &nats).await?;
//                  }
//              }
//          }
//
//          // Create new nodes with configures edges!
//          if let Some(create_list) = configures_result.create {
//              let billing_account_id = entity_json["siStorable"]["billingAccountId"]
//                  .as_str()
//                  .ok_or(EntityError::Missing("siStorable.billingAccountId".into()))?;
//              let organization_id = entity_json["siStorable"]["organizationId"]
//                  .as_str()
//                  .ok_or(EntityError::Missing("siStorable.organizationId".into()))?;
//              let workspace_id = entity_json["siStorable"]["workspaceId"]
//                  .as_str()
//                  .ok_or(EntityError::Missing("siStorable.workspaceId".into()))?;
//              let change_set_id = entity_json["siChangeSet"]["changeSetId"]
//                  .as_str()
//                  .ok_or(EntityError::Missing("siChangeSet.changeSetId".into()))?;
//              let edit_session_id = entity_json["siChangeSet"]["editSessionId"]
//                  .as_str()
//                  .ok_or(EntityError::Missing("siChangeSet.editSessionId".into()))?;
//
//              for to_create in create_list.into_iter() {
//                  trace!(?to_create, "for create list");
//                  if &to_create.object_type == object_type {
//                      trace!(
//                          ?object_type,
//                          ?to_create,
//                          "calculate configures requested an object \
//                          of the same type as this one, which is a recursive thing - skipping it!"
//                      );
//                      continue;
//                  }
//                  let new_node = Node::new(
//                      db.clone(),
//                      nats.clone(),
//                      to_create.name,
//                      NodeKind::Entity,
//                      &to_create.object_type,
//                      billing_account_id.into(),
//                      organization_id.into(),
//                      workspace_id.into(),
//                      change_set_id.into(),
//                      edit_session_id.into(),
//                      None,
//                      Some(to_create.systems.clone()),
//                  )
//                  .await
//                  .map_err(|e| EntityError::Node(e.to_string()))?;
//                  let new_object_id = new_node
//                      .get_object_id(&db)
//                      .await
//                      .map_err(|e| EntityError::Node(e.to_string()))?;
//                  Edge::new(
//                      &db,
//                      &nats,
//                      Vertex::new(node_id, id, "output", object_type),
//                      Vertex::new(&new_node.id, &new_object_id, "input", &new_node.object_type),
//                      false,
//                      EdgeKind::Configures,
//                      billing_account_id.into(),
//                      organization_id.into(),
//                      workspace_id.into(),
//                      None,
//                  )
//                  .await?;
//                  trace!(?new_node, "created node as configured");
//              }
//          }
//
//         Ok(())
//     })
// }
