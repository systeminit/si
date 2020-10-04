use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::HashMap;

use crate::data::{Connection, Db};
use crate::models::{
    calculate_properties, get_base_object, get_model, insert_model, insert_model_if_missing, ops,
    upsert_model, Entity, EntityError, ModelError, SiChangeSetEvent, SiStorable, SiStorableError,
    SimpleStorable, System,
};

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("malformed change set entry; type is missing")]
    TypeMissing,
    #[error("malformed change set entry; id is missing")]
    IdMissing,
    #[error("malformed change set entry; to_id is missing")]
    ToIdMissing,
    #[error("data layer error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("unknown op; this is a bug! add it to the dispatch table: {0}")]
    UnknownOp(String),
    #[error("op error: {0}")]
    Op(#[from] ops::OpError),
    #[error("error creating our object from json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("error in entity: {0}")]
    Entity(#[from] EntityError),
    #[error("missing head value in object")]
    MissingHead,
    #[error("missing change set event field")]
    EventMissing,
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: Option<String>,
    pub organization_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: ChangeSet,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchRequest {
    pub op: PatchOps,
    pub organization_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchOps {
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
        nats: &Connection,
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
            Some(created_by_user_id),
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
        insert_model(db, nats, &change_set.id, &change_set).await?;
        Ok(change_set)
    }

    pub async fn get(
        db: &Db,
        change_set_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> ChangeSetResult<ChangeSet> {
        let change_set_id = change_set_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let change_set: ChangeSet = get_model(db, change_set_id, billing_account_id).await?;
        Ok(change_set)
    }

    #[tracing::instrument(level = "info")]
    pub async fn execute(
        &mut self,
        db: &Db,
        nats: &Connection,
        hypothetical: bool,
    ) -> ChangeSetResult<Vec<String>> {
        let change_set_entry_query = format!(
            "SELECT a.*
          FROM `{bucket}` AS a
          WHERE a.siChangeSet.changeSetId = $change_set_id 
            AND (a.siChangeSet.event = \"Operation\" OR a.siChangeSet.event = \"Delete\" OR a.siChangeSet.event = \"Create\")
          ORDER BY a.siChangeSet.orderClock.epoch ASC, a.siChangeSet.orderClock.updateCount ASC
        ",
            bucket = db.bucket_name,
        );
        let mut change_set_entry_named_params: HashMap<String, serde_json::Value> = HashMap::new();
        change_set_entry_named_params.insert("change_set_id".into(), serde_json::json![&self.id]);
        let change_set_entry_query_results: Vec<serde_json::Value> = db
            .query_consistent(change_set_entry_query, Some(change_set_entry_named_params))
            .await?;

        tracing::error!(?change_set_entry_query_results, "change set exec results");

        let mut seen_map: HashMap<String, serde_json::Value> = HashMap::new();
        let mut last_id: Option<String> = None;
        let mut last_type_name: Option<String> = None;

        for change_set_entry in change_set_entry_query_results.into_iter() {
            tracing::error!(?change_set_entry, "entry");
            let change_set_event = change_set_entry["siChangeSet"]["event"]
                .as_str()
                .ok_or(ChangeSetError::EventMissing)?;
            let entry_type_name = change_set_entry["siStorable"]["typeName"]
                .as_str()
                .ok_or(ChangeSetError::TypeMissing)?;
            let to_id = match change_set_entry["toId"].as_str() {
                Some(to_id) => to_id,
                None => change_set_entry["siStorable"]["objectId"]
                    .as_str()
                    .ok_or(ChangeSetError::ToIdMissing)?,
            };

            if last_id.is_some() {
                let lei = last_id.as_ref().unwrap();
                tracing::warn!(?lei, ?to_id, "comparing lei to to_id");
                if lei != to_id {
                    let last_obj = seen_map.get_mut(last_id.as_ref().unwrap()).unwrap();
                    let last_obj_type_name = last_obj["siStorable"]["typeName"]
                        .as_str()
                        .ok_or(ChangeSetError::TypeMissing)?;
                    if last_obj_type_name == "entity" {
                        calculate_properties(last_obj).await?;
                    }
                }
            };

            let obj = if seen_map.contains_key(to_id) {
                seen_map.get_mut(to_id).unwrap()
            } else {
                tracing::error!("getting base object: {} {}", to_id, &self.id);
                let head_obj = get_base_object(db, to_id, &self.id).await?;
                seen_map.insert(String::from(to_id), head_obj);
                seen_map.get_mut(to_id).unwrap()
            };
            let obj_type_name = obj["siStorable"]["typeName"]
                .as_str()
                .ok_or(ChangeSetError::TypeMissing)?;
            last_type_name = Some(String::from(obj_type_name));

            last_id = Some(String::from(to_id));

            match change_set_event {
                "Create" => {
                    tracing::error!("creating some shit");
                }
                "Operation" => match entry_type_name {
                    "opEntitySet" => {
                        let op: ops::OpEntitySet = serde_json::from_value(change_set_entry)?;
                        tracing::warn!(?op, "applying op");
                        op.apply(obj).await?;
                    }
                    "opSetName" => {
                        tracing::warn!(?change_set_entry, "about to deserialize");
                        let op: ops::OpSetName = serde_json::from_value(change_set_entry)?;
                        tracing::warn!(?op, "applying op");
                        op.apply(obj).await?;
                    }
                    "opEntityDelete" => {
                        let op: ops::OpEntityDelete = serde_json::from_value(change_set_entry)?;
                        tracing::warn!(?op, "applying op");
                        op.apply(obj).await?;
                    }
                    "opEntityAction" => {
                        let op: ops::OpEntityAction = serde_json::from_value(change_set_entry)?;
                        tracing::warn!(?op, "applying op");
                        op.apply(&db, &nats, hypothetical, obj).await?;
                    }
                    unknown => tracing::error!("cannot find an op for {}", unknown),
                },
                unknown => tracing::error!("unknkown change set event {}", unknown),
            }
        }

        //// Calculate the final entities properties
        tracing::warn!(?last_type_name, "what is the last type name");
        if last_type_name.is_some() && last_type_name.unwrap() == "entity" {
            tracing::warn!("why aren't you getting here");
            let mut last_entity = seen_map.get_mut(&last_id.unwrap()).unwrap();
            calculate_properties(&mut last_entity).await?;
        }

        // Now save all the new representations. If it is a hypothetical execution,
        // then save all the models to thier changeSet views. If it is not hypothetical, then save
        // their changeSet views *and* their final form, updating the head bit.
        for (id, obj) in seen_map.iter_mut() {
            if hypothetical {
                let projection_id = format!("{}:{}", id, &self.id);
                {
                    let head = obj
                        .pointer_mut("/head")
                        .ok_or(ChangeSetError::MissingHead)?;
                    *head = serde_json::Value::Bool(false);
                    let change_set_id = obj
                        .pointer_mut("/siChangeSet/changeSetId")
                        .ok_or(ChangeSetError::IdMissing)?;
                    *change_set_id = serde_json::Value::String(String::from(&self.id));
                    let edit_session_id = obj
                        .pointer_mut("/siChangeSet/editSessionId")
                        .ok_or(ChangeSetError::IdMissing)?;
                    *edit_session_id = serde_json::Value::String(String::from(""));
                    let change_set_event = obj
                        .pointer_mut("/siChangeSet/event")
                        .ok_or(ChangeSetError::EventMissing)?;
                    *change_set_event = serde_json::json![SiChangeSetEvent::Projection];
                }
                upsert_model(db, nats, projection_id, obj).await?;
            } else {
                let projection_id = format!("{}:{}", id, &self.id);
                {
                    let head = obj
                        .pointer_mut("/head")
                        .ok_or(ChangeSetError::MissingHead)?;
                    *head = serde_json::Value::Bool(true);
                }
                upsert_model(db, nats, id, obj).await?;
                {
                    let head = obj
                        .pointer_mut("/head")
                        .ok_or(ChangeSetError::MissingHead)?;
                    *head = serde_json::Value::Bool(false);
                    let change_set_id = obj
                        .pointer_mut("/siChangeSet/changeSetId")
                        .ok_or(ChangeSetError::IdMissing)?;
                    *change_set_id = serde_json::Value::String(String::from(&self.id));
                    let edit_session_id = obj
                        .pointer_mut("/siChangeSet/editSessionId")
                        .ok_or(ChangeSetError::IdMissing)?;
                    *edit_session_id = serde_json::Value::String(String::from(""));
                    let change_set_event = obj
                        .pointer_mut("/siChangeSet/event")
                        .ok_or(ChangeSetError::EventMissing)?;
                    *change_set_event = serde_json::json![SiChangeSetEvent::Projection];
                }
                upsert_model(db, nats, projection_id, obj).await?;
            }
        }

        let response: Vec<String> = seen_map.keys().map(|k| String::from(k)).collect();

        if !hypothetical {
            self.status = ChangeSetStatus::Closed;
            upsert_model(&db, &nats, &self.id, &self).await?;
        }

        Ok(response)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetParticipant {
    pub id: String,
    pub change_set_id: String,
    pub object_id: String,
    pub si_storable: SimpleStorable,
}

// in to this entity, and making sure they all have change set participant records as well.
impl ChangeSetParticipant {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        change_set_id: impl Into<String>,
        object_id: impl Into<String>,
        billing_account_id: String,
    ) -> ChangeSetResult<(ChangeSetParticipant, bool)> {
        let change_set_id = change_set_id.into();
        let object_id = object_id.into();
        let id = format!("{}:{}", &change_set_id, &object_id);

        let si_storable = SimpleStorable::new(&id, "changeSetParticipant", billing_account_id);
        let change_set_participant = ChangeSetParticipant {
            id,
            change_set_id,
            object_id,
            si_storable,
        };
        let inserted = insert_model_if_missing(
            db,
            nats,
            &change_set_participant.id,
            &change_set_participant,
        )
        .await?;
        Ok((change_set_participant, inserted))
    }
}
