use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{trace, warn};

use std::collections::HashMap;

use crate::data::{Connection, Db};
use crate::models::{
    calculate_properties, get_base_object, get_model, insert_model, insert_model_if_missing, ops,
    upsert_model, Edge, EdgeError, EdgeKind, Entity, EntityError, ModelError, SiChangeSetEvent,
    SiStorable, SiStorableError, SimpleStorable,
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
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
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

        trace!(?change_set_entry_query_results, "change set exec results");

        let mut seen_map: HashMap<String, serde_json::Value> = HashMap::new();
        let mut last_id: Option<String> = None;
        let mut last_type_name: Option<String> = None;

        for change_set_entry in change_set_entry_query_results.into_iter() {
            trace!(?change_set_entry, "entry");
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
                trace!(?lei, ?to_id, "comparing lei to to_id");
                if lei != to_id {
                    // TODO: This is a pretty needless copy, but it makes it work
                    // for now.
                    let mc = seen_map.clone();
                    let last_obj = seen_map.get_mut(last_id.as_ref().unwrap()).unwrap();
                    let last_obj_type_name = last_obj["siStorable"]["typeName"]
                        .as_str()
                        .ok_or(ChangeSetError::TypeMissing)?;
                    if last_obj_type_name == "entity" {
                        calculate_properties(db, last_obj, Some(&mc)).await?;
                    }
                }
            };

            let mc = seen_map.clone();
            let obj = if seen_map.contains_key(to_id) {
                seen_map.get_mut(to_id).unwrap()
            } else {
                trace!("getting base object: {} {}", to_id, &self.id);
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
                    let type_name = obj["siStorable"]["typeName"]
                        .as_str()
                        .ok_or(ChangeSetError::TypeMissing)?;
                    if type_name == "entity" {
                        calculate_properties(db, obj, Some(&mc)).await?;
                    }
                }
                "Operation" => match entry_type_name {
                    "opEntitySet" => {
                        let op: ops::OpEntitySet = serde_json::from_value(change_set_entry)?;
                        trace!(?op, "applying op");
                        op.apply(obj).await?;
                    }
                    "opSetName" => {
                        trace!(?change_set_entry, "about to deserialize");
                        let op: ops::OpSetName = serde_json::from_value(change_set_entry)?;
                        trace!(?op, "applying op");
                        op.apply(obj).await?;
                    }
                    "opEntityDelete" => {
                        let op: ops::OpEntityDelete = serde_json::from_value(change_set_entry)?;
                        trace!(?op, "applying op");
                        op.apply(obj).await?;
                    }
                    "opEntityAction" => {
                        let op: ops::OpEntityAction = serde_json::from_value(change_set_entry)?;
                        trace!(?op, "applying op");
                        op.apply(&db, &nats, hypothetical, obj).await?;
                    }
                    unknown => warn!("cannot find an op for {}", unknown),
                },
                unknown => warn!("unknkown change set event {}", unknown),
            }
        }

        if last_type_name.is_some() && last_type_name.unwrap() == "entity" {
            let mc = seen_map.clone();
            let mut last_entity = seen_map.get_mut(&last_id.unwrap()).unwrap();
            calculate_properties(db, &mut last_entity, Some(&mc)).await?;
        }

        // Now that we have reached the end of the changeset, we need to
        // calculate the properties of every successor of every node in
        // the change set.
        //
        // This is the most brute-force way possible - I'm 100% certain we can
        // be dramatically more efficient, but this is going to work.
        //
        // There may be a bug here - we might actually need to do this any time we
        // calculate properties in the core loop, to make sure all the values are
        // right for any action we're about to take.
        let seen_map_keys: Vec<String> = seen_map.keys().map(|k| k.clone()).collect();
        let mut processed_list: Vec<String> = vec![];
        for parent_id in seen_map_keys {
            let successor_edges =
                Edge::all_successor_edges_by_object_id(&db, EdgeKind::Configures, &parent_id)
                    .await?;
            for edge in successor_edges.iter() {
                if !processed_list.contains(&edge.head_vertex.object_id) {
                    if seen_map.contains_key(&edge.head_vertex.object_id) {
                        let mc = seen_map.clone();
                        let mut entity_json =
                            seen_map.get_mut(&edge.head_vertex.object_id).unwrap();
                        processed_list.push(edge.head_vertex.object_id.clone());
                        calculate_properties(db, &mut entity_json, Some(&mc)).await?;
                    } else {
                        let entity =
                            Entity::get_projection(&db, &edge.head_vertex.object_id, &self.id)
                                .await?;
                        let entity_id = entity.id.clone();
                        let mut entity_json = serde_json::to_value(entity)?;
                        processed_list.push(edge.head_vertex.object_id.clone());
                        calculate_properties(db, &mut entity_json, Some(&seen_map)).await?;
                        seen_map.insert(entity_id, entity_json);
                    }
                }
            }
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
                    let base = obj
                        .pointer_mut("/base")
                        .ok_or(ChangeSetError::MissingHead)?;
                    *base = serde_json::Value::Bool(false);
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
                    let base = obj
                        .pointer_mut("/base")
                        .ok_or(ChangeSetError::MissingHead)?;
                    *base = serde_json::Value::Bool(false);
                }
                upsert_model(db, nats, id, obj).await?;
                {
                    let head = obj
                        .pointer_mut("/head")
                        .ok_or(ChangeSetError::MissingHead)?;
                    *head = serde_json::Value::Bool(false);
                    let base = obj
                        .pointer_mut("/base")
                        .ok_or(ChangeSetError::MissingHead)?;
                    *base = serde_json::Value::Bool(false);
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
