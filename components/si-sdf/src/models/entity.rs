use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;
use tracing::{error, info, trace};

use crate::data::{Connection, Db, REQWEST};
use crate::models::{
    get_base_object, insert_model, secret::EncryptedSecret, upsert_model, Edge, EdgeError,
    EdgeKind, ModelError, Node, NodeKind, Resource, ResourceError, SecretError, SiChangeSet,
    SiChangeSetError, SiChangeSetEvent, SiStorable, SiStorableError, System, SystemError, Vertex,
};

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
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
    Json(#[from] serde_json::Error),
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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

//pub struct EntityProperties {
//    baseline: serde_json::Value,
//}
//
//impl EntityProperties {
//    pub fn new() -> Self {
//        EntityProperties {
//            baseline: serde_json::json![{}],
//        }
//    }
//}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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
    #[tracing::instrument(level = "trace")]
    pub fn new(
        db: Db,
        nats: Connection,
        name: Option<String>,
        description: Option<String>,
        node_id: String,
        object_type: String,
        head: bool,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        change_set_id: String,
        edit_session_id: String,
        created_by_user_id: Option<String>,
        system_ids: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = EntityResult<Entity>> + Send>> {
        Box::pin(async move {
            if system_ids.len() == 0 {
                return Err(EntityError::NotEnoughSystems);
            }
            let name = crate::models::generate_name(name);
            let description = if description.is_some() {
                description.unwrap()
            } else {
                name.clone()
            };
            let si_storable = SiStorable::new(
                &db,
                "entity",
                billing_account_id.clone(),
                organization_id.clone(),
                workspace_id.clone(),
                created_by_user_id.clone(),
            )
            .await?;
            let expression_properties = EntityProperties::new();
            let manual_properties = EntityProperties::new();
            let inferred_properties = EntityProperties::new();
            let properties = EntityProperties::new();
            let id = si_storable.object_id.clone();
            let key = format!("{}:{}", &si_storable.object_id, &change_set_id);
            let base_key = format!("{}:{}:base", &si_storable.object_id, &change_set_id);
            let si_change_set = SiChangeSet::new(
                &db,
                &nats,
                change_set_id,
                edit_session_id,
                &id,
                billing_account_id.clone(),
                SiChangeSetEvent::Create,
            )
            .await?;
            let mut entity = Entity {
                id: id.clone(),
                name,
                object_type,
                head,
                base: false,
                description,
                expression_properties,
                manual_properties,
                inferred_properties,
                properties,
                node_id: node_id.clone(),
                si_storable,
                si_change_set: Some(si_change_set),
            };

            insert_model(&db, &nats, &key, &entity).await?;
            entity.base = true;
            insert_model(&db, &nats, &base_key, &entity).await?;
            for system_id in system_ids {
                trace!(?system_id, ?entity, "getting system edge");
                let system = System::get_any(&db, &system_id).await?;
                trace!(?system_id, ?system, ?entity, "adding sytem edge");
                Edge::new(
                    &db,
                    &nats,
                    Vertex::new(&system.node_id, &system.id, "output", "system"),
                    Vertex::new(&entity.node_id, &entity.id, "input", &entity.object_type),
                    false,
                    EdgeKind::Includes,
                    billing_account_id.clone(),
                    organization_id.clone(),
                    workspace_id.clone(),
                    None,
                )
                .await?;
                Resource::new(
                    &db,
                    &nats,
                    serde_json::json![{}],
                    &system_id,
                    &node_id,
                    &id,
                    billing_account_id.clone(),
                    organization_id.clone(),
                    workspace_id.clone(),
                    created_by_user_id.clone(),
                )
                .await?;
            }
            entity.calculate_properties(&db).await?;
            upsert_model(&db, &nats, &key, &entity).await?;
            entity.base = true;
            upsert_model(&db, &nats, &base_key, &entity).await?;

            Ok(entity)
        })
    }

    pub fn calculate_configures(
        &self,
        db: Db,
        nats: Connection,
    ) -> Pin<Box<dyn Future<Output = EntityResult<()>> + Send>> {
        let entity_json = serde_json::json![self];
        Box::pin(async move {
            calculate_configures(db, nats, entity_json).await?;
            Ok(())
        })
    }

    pub async fn calculate_properties(&mut self, db: &Db) -> EntityResult<()> {
        let mut json = serde_json::json![self];
        calculate_properties(db, &mut json, None).await?;
        let new_entity: Entity = serde_json::from_value(json)?;
        trace!(?new_entity, "new entity from calculate properties");
        *self = new_entity;
        Ok(())
    }

    pub async fn update_properties_if_secret(&mut self, db: &Db) -> EntityResult<()> {
        if let Some(secret_id) = self
            .properties
            .get_property("/secretId", None)?
            .map(|s| s.as_str())
            .flatten()
        {
            let secret =
                EncryptedSecret::get(db, secret_id, &self.si_storable.billing_account_id).await?;
            let decrypted = secret.decrypt(db).await?;
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

    pub async fn get_projection(
        db: &Db,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        let entity_id = entity_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"entity\"
            AND a.siStorable.objectId = $entity_id 
            AND a.siChangeSet.changeSetId = $change_set_id
            AND a.head = false
          ORDER BY a.base ASC
          LIMIT 1
        ",
            bucket = db.bucket_name
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("entity_id".into(), serde_json::json![entity_id]);
        named_params.insert("change_set_id".into(), serde_json::json![change_set_id]);
        let mut query_results: Vec<Entity> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(EntityError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get_head(db: &Db, entity_id: impl AsRef<str>) -> EntityResult<Entity> {
        let entity_id = entity_id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"entity\"
            AND a.siStorable.objectId = $entity_id 
            AND a.head = true
          LIMIT 1
        ",
            bucket = db.bucket_name
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("entity_id".into(), serde_json::json![entity_id]);
        let mut query_results: Vec<Entity> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(EntityError::NoHead)
        } else {
            let result = query_results.pop().unwrap();
            Ok(result)
        }
    }

    pub async fn get_projection_or_head(
        db: &Db,
        entity_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
    ) -> EntityResult<Entity> {
        match Self::get_projection(db, &entity_id, change_set_id).await {
            Ok(entity) => Ok(entity),
            Err(EntityError::NoHead) => Self::get_head(db, entity_id).await,
            Err(e) => Err(e),
        }
    }

    pub async fn get_all(db: &Db, entity_id: impl AsRef<str>) -> EntityResult<Vec<Entity>> {
        let entity_id = entity_id.as_ref();
        let query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siStorable.typeName = \"entity\"
            AND a.siStorable.objectId = $entity_id 
        ",
            bucket = db.bucket_name
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("entity_id".into(), serde_json::json![entity_id]);
        let query_results: Vec<Entity> = db.query(query, Some(named_params)).await?;
        if query_results.len() == 0 {
            Err(EntityError::NotFound)
        } else {
            Ok(query_results)
        }
    }

    pub async fn get_any(db: &Db, entity_id: impl AsRef<str>) -> EntityResult<Entity> {
        let entity_id = entity_id.as_ref();
        let mut entity_list = Entity::get_all(db, entity_id).await?;
        entity_list.pop().ok_or(EntityError::NoHead)
    }
}

pub async fn calculate_properties(
    db: &Db,
    json: &mut serde_json::Value,
    projections: Option<&HashMap<String, serde_json::Value>>,
) -> EntityResult<()> {
    info!(?json, "calculating properties");
    let entity: Entity = serde_json::from_value(json.clone())?;
    let optional_change_set_id = if entity.head {
        None
    } else {
        entity.si_change_set.map(|sic| sic.change_set_id.clone())
    };
    let node = Node::get(&db, entity.node_id, &entity.si_storable.billing_account_id)
        .await
        .map_err(|e| EntityError::Node(e.to_string()))?;
    let system_edges = Edge::by_kind_and_head_object_id_and_tail_type_name(
        &db,
        EdgeKind::Includes,
        &entity.id,
        "system",
    )
    .await?;

    let mut resources = Vec::new();
    for system_edge in system_edges.iter() {
        trace!(?system_edge, "system edge");
        let resource = Resource::get(&db, &entity.id, &system_edge.tail_vertex.object_id).await?;
        resources.push(resource);
    }

    let predecessor_edges =
        Edge::direct_predecessor_edges_by_node_id(db, EdgeKind::Configures, &node.id).await?;
    let mut predecessors: Vec<CalculatePropertiesPredecessor> = Vec::new();
    for edge in predecessor_edges {
        let edge_node = Node::get(
            &db,
            &edge.tail_vertex.node_id,
            &node.si_storable.billing_account_id,
        )
        .await
        .map_err(|e| EntityError::Node(e.to_string()))?;
        // OMG, I'm so sorry
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
                match edge_node.get_object_projection(&db, change_set_id).await {
                    Ok(entity) => entity,
                    Err(_) => edge_node
                        .get_head_object(db)
                        .await
                        .map_err(|e| EntityError::Node(e.to_string()))?,
                }
            } else if let Ok(entity) = edge_node.get_head_object(db).await {
                entity
            } else {
                return Err(EntityError::Node("no head node!".to_string()));
            }
        };
        let mut edge_resources: Vec<Resource> = Vec::new();
        trace!(?system_edges, "calculating edge resources for system edges");
        for system_edge in system_edges.iter() {
            let edge_resource =
                Resource::get(&db, &edge_entity.id, &system_edge.tail_vertex.object_id).await?;
            trace!(?edge_resource, "no mas");
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
    let res = REQWEST
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

pub fn calculate_configures(
    db: Db,
    nats: Connection,
    entity_json: serde_json::Value,
) -> Pin<Box<dyn Future<Output = EntityResult<()>> + Send>> {
    Box::pin(async move {
        let id = entity_json["id"].as_str().ok_or(EntityError::MissingId)?;
        let object_type = entity_json["objectType"]
            .as_str()
            .ok_or(EntityError::MissingObjectType)?;
        let node_id = entity_json["nodeId"]
            .as_str()
            .ok_or(EntityError::Missing("nodeId".into()))?;
        let change_set_id = entity_json["siChangeSet"]["changeSetId"]
            .as_str()
            .unwrap_or("fakemcfakerton");

        // Get the list of edges this entity configures
        let configures_edges =
            Edge::by_kind_and_tail_object_id(&db, EdgeKind::Configures, id).await?;
        let mut configures = Vec::with_capacity(configures_edges.len());
        for edge in configures_edges.iter() {
            let object = get_base_object(&db, &edge.head_vertex.object_id, change_set_id).await?;
            configures.push(object);
        }

        // Get the list of systems this entity participates in
        let system_edges = Edge::by_kind_and_head_object_id_and_tail_type_name(
            &db,
            EdgeKind::Includes,
            id,
            "system",
        )
        .await?;
        let mut systems: Vec<System> = Vec::with_capacity(system_edges.len());
        for system_edge in system_edges.iter() {
            let system = System::get_any(&db, &system_edge.tail_vertex.object_id).await?;
            systems.push(system);
        }
        trace!(?systems, ?node_id, "making nodes with the list of systems");

        let res = REQWEST
            .post("http://localhost:5157/calculateConfigures")
            .json(&CalculateConfiguresRequest {
                entity: &entity_json,
                configures: &serde_json::json![configures],
                systems: &serde_json::json![systems],
            })
            .send()
            .await?;
        let configures_result: CalculateConfiguresResponse = res.json().await?;

        // If any edge is not in the keep list from the callback, then we remove its
        // connection.
        if let Some(keep) = configures_result.keep {
            for edge in configures_edges.into_iter() {
                if !keep.contains(&edge.head_vertex.object_id) {
                    edge.delete(&db, &nats).await?;
                }
            }
        }

        // Create new nodes with configures edges!
        if let Some(create_list) = configures_result.create {
            let billing_account_id = entity_json["siStorable"]["billingAccountId"]
                .as_str()
                .ok_or(EntityError::Missing("siStorable.billingAccountId".into()))?;
            let organization_id = entity_json["siStorable"]["organizationId"]
                .as_str()
                .ok_or(EntityError::Missing("siStorable.organizationId".into()))?;
            let workspace_id = entity_json["siStorable"]["workspaceId"]
                .as_str()
                .ok_or(EntityError::Missing("siStorable.workspaceId".into()))?;
            let change_set_id = entity_json["siChangeSet"]["changeSetId"]
                .as_str()
                .ok_or(EntityError::Missing("siChangeSet.changeSetId".into()))?;
            let edit_session_id = entity_json["siChangeSet"]["editSessionId"]
                .as_str()
                .ok_or(EntityError::Missing("siChangeSet.editSessionId".into()))?;

            for to_create in create_list.into_iter() {
                trace!(?to_create, "for create list");
                if &to_create.object_type == object_type {
                    trace!(
                        ?object_type,
                        ?to_create,
                        "calculate configures requested an object \
                        of the same type as this one, which is a recursive thing - skipping it!"
                    );
                    continue;
                }
                let new_node = Node::new(
                    db.clone(),
                    nats.clone(),
                    to_create.name,
                    NodeKind::Entity,
                    &to_create.object_type,
                    billing_account_id.into(),
                    organization_id.into(),
                    workspace_id.into(),
                    change_set_id.into(),
                    edit_session_id.into(),
                    None,
                    Some(to_create.systems.clone()),
                )
                .await
                .map_err(|e| EntityError::Node(e.to_string()))?;
                let new_object_id = new_node
                    .get_object_id(&db)
                    .await
                    .map_err(|e| EntityError::Node(e.to_string()))?;
                Edge::new(
                    &db,
                    &nats,
                    Vertex::new(node_id, id, "output", object_type),
                    Vertex::new(&new_node.id, &new_object_id, "input", &new_node.object_type),
                    false,
                    EdgeKind::Configures,
                    billing_account_id.into(),
                    organization_id.into(),
                    workspace_id.into(),
                    None,
                )
                .await?;
                trace!(?new_node, "created node as configured");
            }
        }

        Ok(())
    })
}
