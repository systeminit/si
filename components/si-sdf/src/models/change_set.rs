use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;
use tracing::{trace, warn};

use std::collections::HashMap;

use crate::data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};
use crate::models::{
    calculate_properties, list_model, next_update_clock, ops, Edge, EdgeError, EdgeKind, Entity,
    EntityError, Event, EventError, ListReply, ModelError, OrderByDirection, PageToken, Query,
    SiChangeSet, SiChangeSetEvent, SiStorable, System, SystemError, UpdateClock, UpdateClockError,
};
use crate::veritech::Veritech;

const CHANGE_SET_PARTICIPANT_EXISTS: &str =
    include_str!("../data/queries/change_set_participant_exists.sql");
const CHANGE_SET_ENTRIES: &str = include_str!("../data/queries/change_set_entries.sql");

#[derive(Error, Debug)]
pub enum ChangeSetError {
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
    #[error("event error: {0}")]
    Event(#[from] EventError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("unknown change-set aware object; should be entity or system")]
    UnknownObjectType,
    #[error("system error: {0}")]
    System(#[from] SystemError),
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
    ExecuteWithAction(ExecuteWithActionRequest),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequest {
    pub hypothetical: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWithActionRequest {
    pub node_id: String,
    pub action: String,
    pub system_id: String,
    pub edit_session_id: String,
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

#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ChangeSetStatus {
    Open,
    Closed,
    Abandoned,
    Executing,
    Failed,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: Option<String>,
        workspace_id: String,
    ) -> ChangeSetResult<ChangeSet> {
        let name = crate::models::generate_name(name);
        let update_clock = next_update_clock(&workspace_id).await?;
        let row = txn
            .query_one(
                "SELECT object FROM change_set_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    &name,
                    &String::new(),
                    &ChangeSetStatus::Open.to_string(),
                    &workspace_id,
                    &update_clock.epoch,
                    &update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ChangeSet = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
    ) -> ChangeSetResult<ChangeSet> {
        let id = change_set_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM change_set_get_v1($1)", &[&id])
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
    ) -> ChangeSetResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "change_sets",
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

    pub async fn execute(
        &mut self,
        pg: &PgPool,
        txn: &PgTxn<'_>,
        nats_conn: &NatsConn,
        nats: &NatsTxn,
        veritech: &Veritech,
        hypothetical: bool,
        parent_event_id: Option<&str>,
    ) -> ChangeSetResult<Vec<String>> {
        let rows = txn.query(CHANGE_SET_ENTRIES, &[&self.id]).await?;
        let mut change_set_entry_query_results: Vec<serde_json::Value> = vec![];
        for row in rows.into_iter() {
            let json = row.try_get("object")?;
            change_set_entry_query_results.push(json);
        }

        warn!(?change_set_entry_query_results, "change set exec results");

        let event =
            Event::change_set_execute(&pg, &nats_conn, &self, parent_event_id.map(|id| id.into()))
                .await?;

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
                        calculate_properties(&txn, last_obj, Some(&mc)).await?;
                    }
                }
            };

            let mc = seen_map.clone();
            let obj = if seen_map.contains_key(to_id) {
                seen_map.get_mut(to_id).unwrap()
            } else {
                let head_obj = match entry_type_name {
                    "entity" => {
                        let entity = Entity::get_head_or_base(&txn, to_id, &self.id).await?;
                        serde_json::to_value(entity)?
                    }
                    "system" => {
                        let system = System::get_head_or_base(&txn, to_id, &self.id).await?;
                        serde_json::to_value(system)?
                    }
                    // Will we hit a bug here when you get an Op? Lets find out?
                    o => {
                        if to_id.starts_with("entity:") {
                            let entity = Entity::get_head_or_base(&txn, to_id, &self.id).await?;
                            serde_json::to_value(entity)?
                        } else if to_id.starts_with("system:") {
                            let system = System::get_head_or_base(&txn, to_id, &self.id).await?;
                            serde_json::to_value(system)?
                        } else {
                            warn!(?o, "target object is not an entity or a system, that's bad");
                            return Err(ChangeSetError::UnknownObjectType);
                        }
                    }
                };
                seen_map.insert(String::from(to_id), head_obj);
                seen_map.get_mut(to_id).unwrap()
            };
            let obj_type_name = obj["siStorable"]["typeName"]
                .as_str()
                .ok_or(ChangeSetError::TypeMissing)?;
            last_type_name = Some(String::from(obj_type_name));

            last_id = Some(String::from(to_id));

            match change_set_event {
                "create" => {
                    let type_name = obj["siStorable"]["typeName"]
                        .as_str()
                        .ok_or(ChangeSetError::TypeMissing)?;
                    if type_name == "entity" {
                        calculate_properties(&txn, obj, Some(&mc)).await?;
                    }
                }
                "operation" => match entry_type_name {
                    "opEntitySet" => {
                        let op: ops::OpEntitySet = serde_json::from_value(change_set_entry)?;
                        trace!(?op, "applying op");
                        op.apply(obj).await?;
                    }
                    "opSetName" => {
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
                        op.apply(
                            &pg,
                            &txn,
                            &nats_conn,
                            &veritech,
                            hypothetical,
                            obj,
                            Some(event.id.clone()),
                        )
                        .await?;
                    }
                    unknown => warn!("cannot find an op for {}", unknown),
                },
                unknown => {
                    warn!("unknkown change set event {}", unknown)
                }
            }
        }

        if last_type_name.is_some() && last_type_name.unwrap() == "entity" {
            let mc = seen_map.clone();
            let mut last_entity = seen_map.get_mut(&last_id.unwrap()).unwrap();
            calculate_properties(&txn, &mut last_entity, Some(&mc)).await?;
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
                Edge::all_successor_edges_by_object_id(&txn, &EdgeKind::Configures, &parent_id)
                    .await?;
            for edge in successor_edges.iter() {
                if !processed_list.contains(&edge.head_vertex.object_id) {
                    if seen_map.contains_key(&edge.head_vertex.object_id) {
                        let mc = seen_map.clone();
                        let mut entity_json =
                            seen_map.get_mut(&edge.head_vertex.object_id).unwrap();
                        processed_list.push(edge.head_vertex.object_id.clone());
                        calculate_properties(&txn, &mut entity_json, Some(&mc)).await?;
                    } else {
                        let entity = Entity::get_projection_or_head(
                            &txn,
                            &edge.head_vertex.object_id,
                            &self.id,
                        )
                        .await?;
                        let entity_id = entity.id.clone();
                        let mut entity_json = serde_json::to_value(entity)?;
                        processed_list.push(edge.head_vertex.object_id.clone());
                        calculate_properties(&txn, &mut entity_json, Some(&seen_map)).await?;
                        seen_map.insert(entity_id, entity_json);
                    }
                }
            }
        }

        // Now save all the new representations. If it is a hypothetical execution,
        // then save all the models to thier changeSet views. If it is not hypothetical, then save
        // their changeSet views *and* their final form, updating the head bit.
        for (_id, obj) in seen_map.iter_mut() {
            if hypothetical {
                let type_name = obj["siStorable"]["typeName"]
                    .as_str()
                    .ok_or(ChangeSetError::TypeMissing)?;
                match type_name {
                    "entity" => {
                        let mut entity: Entity = serde_json::from_value(obj.clone())?;
                        if entity.si_change_set.is_none() {
                            entity.si_change_set = Some(SiChangeSet {
                                change_set_id: self.id.clone(),
                                edit_session_id: String::new(),
                                event: SiChangeSetEvent::Projection,
                                order_clock: UpdateClock {
                                    epoch: 0,
                                    update_count: 0,
                                },
                            });
                        }
                        entity.save_projection(&txn, &nats).await?;
                    }
                    "system" => {
                        let mut system: System = serde_json::from_value(obj.clone())?;
                        if system.si_change_set.is_none() {
                            system.si_change_set = Some(SiChangeSet {
                                change_set_id: self.id.clone(),
                                edit_session_id: String::new(),
                                event: SiChangeSetEvent::Projection,
                                order_clock: UpdateClock {
                                    epoch: 0,
                                    update_count: 0,
                                },
                            });
                        }
                        system.save_projection(&txn, &nats).await?;
                    }
                    f_type_name => {
                        tracing::error!(?f_type_name, "got an invalid type name saving change set");
                        return Err(ChangeSetError::UnknownObjectType);
                    }
                }
            } else {
                {
                    let type_name = obj["siStorable"]["typeName"]
                        .as_str()
                        .ok_or(ChangeSetError::TypeMissing)?;
                    match type_name {
                        "entity" => {
                            let mut entity: Entity = serde_json::from_value(obj.clone())?;
                            entity.save_head(&txn, &nats).await?;
                        }
                        "system" => {
                            let mut system: System = serde_json::from_value(obj.clone())?;
                            system.save_head(&txn, &nats).await?;
                        }
                        f_type_name => {
                            tracing::error!(
                                ?f_type_name,
                                "got an invalid type name saving change set"
                            );
                            return Err(ChangeSetError::UnknownObjectType);
                        }
                    }
                }
                let type_name = obj["siStorable"]["typeName"]
                    .as_str()
                    .ok_or(ChangeSetError::TypeMissing)?;
                match type_name {
                    "entity" => {
                        let mut entity: Entity = serde_json::from_value(obj.clone())?;
                        if entity.si_change_set.is_none() {
                            entity.si_change_set = Some(SiChangeSet {
                                change_set_id: self.id.clone(),
                                edit_session_id: String::new(),
                                event: SiChangeSetEvent::Projection,
                                order_clock: UpdateClock {
                                    epoch: 0,
                                    update_count: 0,
                                },
                            });
                        }
                        entity.save_projection(&txn, &nats).await?;
                    }
                    "system" => {
                        let mut system: System = serde_json::from_value(obj.clone())?;
                        if system.si_change_set.is_none() {
                            system.si_change_set = Some(SiChangeSet {
                                change_set_id: self.id.clone(),
                                edit_session_id: String::new(),
                                event: SiChangeSetEvent::Projection,
                                order_clock: UpdateClock {
                                    epoch: 0,
                                    update_count: 0,
                                },
                            });
                        }
                        system.save_projection(&txn, &nats).await?;
                    }
                    f_type_name => {
                        tracing::error!(?f_type_name, "got an invalid type name saving change set");
                        return Err(ChangeSetError::UnknownObjectType);
                    }
                }
            }
        }

        let response: Vec<String> = seen_map.keys().map(|k| String::from(k)).collect();

        if !hypothetical {
            self.status = ChangeSetStatus::Closed;
            self.save(&txn, &nats).await?;
        }

        Ok(response)
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> ChangeSetResult<()> {
        let update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = update_clock;

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM change_set_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: ChangeSet = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetParticipant {
    pub id: String,
    pub change_set_id: String,
    pub object_id: String,
    pub si_storable: SiStorable,
}

// in to this entity, and making sure they all have change set participant records as well.
impl ChangeSetParticipant {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        change_set_id: impl AsRef<str>,
        object_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> ChangeSetResult<ChangeSetParticipant> {
        let change_set_id = change_set_id.as_ref();
        let object_id = object_id.as_ref();
        let workspace_id = workspace_id.as_ref();

        let update_clock = next_update_clock(workspace_id).await?;
        let row = txn
            .query_one(
                "SELECT object FROM change_set_participant_create_v1($1, $2, $3, $4, $5)",
                &[
                    &change_set_id,
                    &object_id,
                    &workspace_id,
                    &update_clock.epoch,
                    &update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ChangeSetParticipant = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn new_if_not_exists(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        change_set_id: impl AsRef<str>,
        object_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> ChangeSetResult<Option<ChangeSetParticipant>> {
        if ChangeSetParticipant::exists(&txn, &change_set_id, &object_id).await? {
            Ok(None)
        } else {
            Ok(Some(
                ChangeSetParticipant::new(&txn, &nats, &change_set_id, &object_id, &workspace_id)
                    .await?,
            ))
        }
    }

    pub async fn exists(
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
        object_id: impl AsRef<str>,
    ) -> ChangeSetResult<bool> {
        let change_set_id = change_set_id.as_ref();
        let object_id = object_id.as_ref();
        let rows = txn
            .query(CHANGE_SET_PARTICIPANT_EXISTS, &[&change_set_id, &object_id])
            .await?;
        if rows.len() > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> ChangeSetResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "change_set_participants",
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
