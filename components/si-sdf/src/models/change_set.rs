use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::HashMap;

use crate::data::Db;
use crate::models::{
    entity::ops, get_model, insert_model, upsert_model, Entity, EntityError, ModelError,
    SiStorable, SiStorableError,
};

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("malformed change set entry; type is missing")]
    TypeMissing,
    #[error("data layer error: {0}")]
    Data(#[from] si_data::DataError),
    #[error("unknown op; this is a bug! add it to the dispatch table: {0}")]
    UnknownOp(String),
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("error in entity: {0}")]
    Entity(#[from] EntityError),
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: ChangeSet,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchRequest {
    Execute(ExecuteRequest),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequest {
    pub hypothetical: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchReply {
    Execute(ExecuteReply),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteReply {
    pub item_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChangeSetStatus {
    Open,
    Closed,
    Abandoned,
    Executing,
    Failed,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    pub id: String,
    pub name: String,
    pub note: String,
    pub status: ChangeSetStatus,
    pub si_storable: SiStorable,
}

impl ChangeSet {
    pub async fn new(
        db: &Db,
        name: Option<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: String,
    ) -> ChangeSetResult<ChangeSet> {
        let name = crate::models::generate_name(name);
        let si_storable = SiStorable::new(
            db,
            "changeSet",
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let id = si_storable.object_id.clone();
        let change_set = ChangeSet {
            id,
            name,
            note: "".to_string(),
            status: ChangeSetStatus::Open,
            si_storable,
        };
        insert_model(db, &change_set.id, &change_set).await?;
        Ok(change_set)
    }

    pub async fn get(
        db: &Db,
        change_set_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> ChangeSetResult<ChangeSet> {
        let change_set_id = change_set_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let json_change_set: serde_json::Value = get_model(
            db,
            change_set_id,
            "changeSet",
            String::from(billing_account_id),
            None,
        )
        .await?;
        let change_set: ChangeSet = serde_json::from_value(json_change_set)?;
        Ok(change_set)
    }

    pub async fn execute(&mut self, db: &Db, hypothetical: bool) -> ChangeSetResult<Vec<String>> {
        let change_set_entry_query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siChangeSet.changeSetId = $change_set_id 
            AND (a.siChangeSet.event = \"Operation\" OR a.siChangeSet.event = \"Delete\" OR a.siChangeSet.event = \"Create\")
            AND a.siOp.skip = false
          ORDER BY a.siChangeSet.orderClock.epoch ASC, a.siChangeSet.orderClock.update_count ASC
        ",
            bucket = db.bucket_name,
        );
        let mut change_set_entry_named_params: HashMap<String, serde_json::Value> = HashMap::new();
        change_set_entry_named_params.insert("change_set_id".into(), serde_json::json![&self.id]);
        let mut change_set_entry_query_results: Vec<serde_json::Value> = db
            .query(change_set_entry_query, Some(change_set_entry_named_params))
            .await?;

        let mut entity_map: HashMap<String, Entity> = HashMap::new();
        let mut last_entity_id: Option<String> = None;

        // Apply all the operations, in order!
        for result in change_set_entry_query_results.into_iter() {
            match result["siStorable"]["typeName"].as_str() {
                Some("opSetString") => {
                    let op: ops::OpSetString = serde_json::from_value(result)?;
                    if last_entity_id.is_some() {
                        let lei = last_entity_id.as_ref().unwrap();
                        if (lei != &op.entity_id) {
                            let last_entity = entity_map.get_mut(&last_entity_id.unwrap()).unwrap();
                            last_entity.calculate_properties().await?;
                        }
                    };
                    let entity = if entity_map.contains_key(&op.entity_id) {
                        entity_map.get_mut(&op.entity_id).unwrap()
                    } else {
                        let head_entity = Entity::get_head(db, &op.entity_id).await?;
                        entity_map.insert(op.entity_id.clone(), head_entity);
                        entity_map.get_mut(&op.entity_id).unwrap()
                    };
                    last_entity_id = Some(op.entity_id.clone());
                    op.apply(entity);
                }
                Some(unknown) => return Err(ChangeSetError::UnknownOp(unknown.into())),
                None => return Err(ChangeSetError::TypeMissing),
            }
        }

        // Calculate the final entities properties
        if last_entity_id.is_some() {
            let last_entity = entity_map.get_mut(&last_entity_id.unwrap()).unwrap();
            last_entity.calculate_properties().await?;
        }

        // Now save all the entities new representations. If it is not a hypothetical execution,
        // then save all the models to thier changeSet views. If it is hypothetical, then save
        // their changeSet views *and* their final form, updating the head bit.
        for (entity_id, entity) in entity_map.iter_mut() {
            if hypothetical {
                let projection_id = format!("{}:{}", entity_id, &self.id);
                entity.head = false;
                upsert_model(db, projection_id, entity).await?;
            } else {
                let projection_id = format!("{}:{}", entity_id, &self.id);
                upsert_model(db, projection_id, entity).await?;
                entity.head = true;
                upsert_model(db, entity_id, entity).await?
            }
        }

        let response: Vec<String> = entity_map.keys().map(|k| String::from(k)).collect();

        Ok(response)
    }
}
