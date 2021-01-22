use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use strum_macros::Display;
use thiserror::Error;

use crate::data::{NatsTxn, NatsTxnError, PgTxn};
use crate::models::{
    list_model, next_update_clock, ops::OpEntityAction, ChangeSet, Entity, EventLog, EventLogError,
    EventLogLevel, ListReply, ModelError, Node, OrderByDirection, PageToken, Query, SiStorable,
    UpdateClockError,
};

#[derive(Error, Debug)]
pub enum EventError {
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("eventLog error: {0}")]
    EventLog(#[from] EventLogError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
}

pub type EventResult<T> = Result<T, EventError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EventKind {
    ChangeSetExecute,
    EntityAction,
    NodeEntityCreate,
    ResourceSync,
    CliChangeRun,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EventStatus {
    Unknown,
    Running,
    Success,
    Error,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub message: String,
    pub kind: EventKind,
    pub context: Vec<String>,
    pub status: EventStatus,
    pub parent_id: Option<String>,
    pub start_unix_timestamp: i64,
    pub start_timestamp: String,
    pub end_unix_timestamp: Option<i64>,
    pub end_timestamp: Option<String>,
    pub si_storable: SiStorable,
    pub payload: serde_json::Value,
}

impl Event {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        message: impl Into<String>,
        payload: impl Into<serde_json::Value>,
        kind: EventKind,
        context: Vec<String>,
        parent_id: Option<String>,
        workspace_id: String,
    ) -> EventResult<Event> {
        let message = message.into();
        let payload = payload.into();

        let current_time = Utc::now();
        let start_unix_timestamp = current_time.timestamp_millis();
        let start_timestamp = format!("{}", current_time);

        let update_clock = next_update_clock(&workspace_id).await?;

        let row = txn
            .query_one(
                "SELECT object FROM event_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    &message,
                    &kind.to_string(),
                    &context,
                    &EventStatus::Running.to_string(),
                    &payload,
                    &start_timestamp,
                    &start_unix_timestamp,
                    &workspace_id,
                    &update_clock.epoch,
                    &update_clock.update_count,
                    &parent_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Event = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn from_si_storable(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        message: impl Into<String>,
        payload: impl Into<serde_json::Value>,
        kind: EventKind,
        context: Option<Vec<String>>,
        parent_id: Option<String>,
        si_storable: &SiStorable,
    ) -> EventResult<Event> {
        let mut context_list = si_storable.tenant_ids.clone();
        if let Some(mut extra_context) = context {
            context_list.append(&mut extra_context);
        };
        Event::new(
            &txn,
            &nats,
            message,
            payload,
            kind,
            context_list,
            parent_id,
            si_storable.workspace_id.clone(),
        )
        .await
    }

    pub async fn node_entity_create(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        node: &Node,
        entity: &Entity,
        parent_id: Option<String>,
    ) -> EventResult<Event> {
        let message = format!("created new {} node", node.object_type);
        let payload = serde_json::json!({
            "objectType": node.object_type,
            "name": entity.name,
            "properties": entity.properties,
        });
        let kind = EventKind::NodeEntityCreate;
        let context = vec![entity.id.clone(), node.id.clone()];
        Event::from_si_storable(
            &txn,
            &nats,
            message,
            payload,
            kind,
            Some(context),
            parent_id,
            &node.si_storable,
        )
        .await
    }

    pub async fn sync_resource(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        entity: &Entity,
        system_id: &str,
        event_parent_id: Option<String>,
    ) -> EventResult<Event> {
        let message = format!(
            "synchronizing resource for {} {}",
            entity.object_type, entity.name
        );
        let payload = serde_json::json!({
            "objectType": entity.object_type,
            "name": entity.name,
            "systemId": system_id,
        });
        let kind = EventKind::ResourceSync;
        let context = vec![
            entity.id.clone(),
            entity.node_id.clone(),
            String::from(system_id),
        ];
        Event::from_si_storable(
            &txn,
            &nats,
            message,
            payload,
            kind,
            Some(context),
            event_parent_id,
            &entity.si_storable,
        )
        .await
    }

    pub async fn entity_action(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        op_entity_action: &OpEntityAction,
        entity: &Entity,
        system_id: &str,
        event_parent_id: Option<String>,
    ) -> EventResult<Event> {
        let message = format!(
            "{} on {}[{}]",
            &op_entity_action.action, entity.object_type, entity.name
        );
        let payload = serde_json::json!({
            "objectType": entity.object_type,
            "name": entity.name,
            "action": op_entity_action.action.clone(),
            "systemId": system_id,
        });
        let kind = EventKind::EntityAction;
        let context = vec![entity.node_id.clone(), String::from(system_id)];
        Event::from_si_storable(
            &txn,
            &nats,
            message,
            payload,
            kind,
            Some(context),
            event_parent_id,
            &entity.si_storable,
        )
        .await
    }

    pub async fn change_set_execute(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        change_set: &ChangeSet,
        event_parent_id: Option<String>,
    ) -> EventResult<Event> {
        let message = format!("executing changeSet {}", &change_set.name);
        let payload = serde_json::json!({
            "name": change_set.name,
        });
        let kind = EventKind::ChangeSetExecute;
        Event::from_si_storable(
            &txn,
            &nats,
            message,
            payload,
            kind,
            None,
            event_parent_id,
            &change_set.si_storable,
        )
        .await
    }

    pub async fn cli_change_run(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        entity: &Entity,
        action: &str,
        system_id: &str,
        event_parent_id: Option<String>,
    ) -> EventResult<Event> {
        let message = format!("CLI {} {}[{}]", action, entity.object_type, entity.name);
        let payload = serde_json::json!({
            "objectType": entity.object_type,
            "name": entity.name,
            "systemId": system_id,
        });
        let kind = EventKind::CliChangeRun;
        let context = vec![
            entity.si_storable.billing_account_id.clone(),
            entity.si_storable.workspace_id.clone(),
            entity.si_storable.organization_id.clone(),
            entity.id.clone(),
            entity.node_id.clone(),
            String::from(system_id),
        ];
        Event::from_si_storable(
            &txn,
            &nats,
            message,
            payload,
            kind,
            Some(context),
            event_parent_id,
            &entity.si_storable,
        )
        .await
    }

    pub async fn running(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EventResult<()> {
        self.status = EventStatus::Running;
        self.save(&txn, &nats).await?;
        Ok(())
    }

    pub async fn success(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EventResult<()> {
        self.status = EventStatus::Success;
        self.save(&txn, &nats).await?;
        Ok(())
    }

    pub async fn error(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EventResult<()> {
        self.status = EventStatus::Error;
        self.save(&txn, &nats).await?;
        Ok(())
    }

    pub async fn unknown(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EventResult<()> {
        self.status = EventStatus::Unknown;
        self.save(&txn, &nats).await?;
        Ok(())
    }

    pub async fn log(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        level: EventLogLevel,
        message: impl Into<String>,
        payload: impl Into<serde_json::Value>,
    ) -> EventResult<EventLog> {
        let message = message.into();
        let payload = payload.into();
        let log = EventLog::new(
            &txn,
            &nats,
            message,
            payload,
            level,
            self.id.clone(),
            self.si_storable.workspace_id.clone(),
        )
        .await?;
        Ok(log)
    }

    // This function checks for ancestry, not for "do you have a parent". I'm great
    // at names.
    pub async fn has_parent(
        &self,
        txn: &PgTxn<'_>,
        parent_id: impl AsRef<str>,
    ) -> EventResult<bool> {
        let parent_id = parent_id.as_ref();
        let mut stack = vec![self.clone()];

        while let Some(event) = stack.pop() {
            match event.parent_id.as_ref() {
                Some(my_parent_id) => {
                    if my_parent_id == parent_id {
                        return Ok(true);
                    } else {
                        let parent: Event = Event::get(&txn, my_parent_id).await?;
                        stack.push(parent);
                    }
                }
                None => {
                    return Ok(false);
                }
            }
        }
        Ok(false)
    }

    pub async fn get(txn: &PgTxn<'_>, event_id: impl AsRef<str>) -> EventResult<Event> {
        let id = event_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM event_get_v1($1)", &[&id])
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
    ) -> EventResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "events",
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

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> EventResult<()> {
        let update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = update_clock;

        let json = serde_json::to_value(&self)?;
        let row = txn
            .query_one("SELECT object FROM event_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let mut updated: Self = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }
}
