use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::data::{Connection, Db, REQWEST};
use crate::models::{
    insert_model, ModelError, SiChangeSet, SiChangeSetError, SiChangeSetEvent, SiStorable,
    SiStorableError,
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
    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type EntityResult<T> = Result<T, EntityError>;

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
pub struct CalculatePropertiesRequest<'a> {
    object_type: &'a str,
    entity: &'a serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculatePropertiesResponse {
    entity: serde_json::Value,
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
    pub si_storable: SiStorable,
    pub si_change_set: Option<SiChangeSet>,
}

impl Entity {
    #[tracing::instrument(level = "trace")]
    pub async fn new(
        db: &Db,
        nats: &Connection,
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
        created_by_user_id: String,
    ) -> EntityResult<Entity> {
        let name = crate::models::generate_name(name);
        let description = if description.is_some() {
            description.unwrap()
        } else {
            name.clone()
        };
        let si_storable = SiStorable::new(
            db,
            "entity",
            billing_account_id,
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;
        let expression_properties = EntityProperties::new();
        let manual_properties = EntityProperties::new();
        let inferred_properties = EntityProperties::new();
        let properties = EntityProperties::new();
        let id = si_storable.object_id.clone();
        let key = format!("{}:{}", si_storable.object_id, &change_set_id);
        let si_change_set =
            SiChangeSet::new(db, change_set_id, edit_session_id, SiChangeSetEvent::Create).await?;
        let mut entity = Entity {
            id,
            name,
            object_type,
            head,
            description,
            expression_properties,
            manual_properties,
            inferred_properties,
            properties,
            node_id,
            si_storable,
            si_change_set: Some(si_change_set),
        };
        entity.calculate_properties().await?;
        insert_model(db, nats, &key, &entity).await?;

        Ok(entity)
    }

    pub async fn calculate_properties(&mut self) -> EntityResult<()> {
        let mut json = serde_json::json![self];
        calculate_properties(&mut json).await?;
        let new_entity: Entity = serde_json::from_value(json)?;
        *self = new_entity;
        Ok(())
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
}

pub async fn calculate_properties(json: &mut serde_json::Value) -> EntityResult<()> {
    let object_type = json["objectType"]
        .as_str()
        .ok_or(EntityError::MissingObjectType)?;
    let res = REQWEST
        .post("http://localhost:5157/calculateProperties")
        .json(&CalculatePropertiesRequest {
            object_type,
            entity: json,
        })
        .send()
        .await?;
    let entity_result: CalculatePropertiesResponse = res.json().await?;
    *json = entity_result.entity;
    Ok(())
}
