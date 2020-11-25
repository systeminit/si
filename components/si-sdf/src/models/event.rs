use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{
    insert_model, ops::OpEntityAction, upsert_model, ChangeSet, Entity, EventLog, EventLogError,
    EventLogLevel, ModelError, Node, SiStorable, SiStorableError,
};

use super::get_model;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("eventLog error: {0}")]
    EventLog(#[from] EventLogError),
}

pub type EventResult<T> = Result<T, EventError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EventKind {
    ChangeSetExecute,
    EntityAction,
    NodeEntityCreate,
    ResourceSync,
    CliChangeRun,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
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
        db: &Db,
        nats: &Connection,
        message: impl Into<String>,
        payload: impl Into<serde_json::Value>,
        kind: EventKind,
        context: Vec<String>,
        parent_id: Option<String>,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: Option<String>,
    ) -> EventResult<Event> {
        let message = message.into();
        let payload = payload.into();
        let si_storable = SiStorable::new(
            db,
            "event",
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let id = si_storable.object_id.clone();

        let current_time = Utc::now();
        let start_unix_timestamp = current_time.timestamp_millis();
        let start_timestamp = format!("{}", current_time);

        let event = Event {
            id,
            message,
            start_unix_timestamp,
            start_timestamp,
            kind,
            context,
            payload,
            status: EventStatus::Running,
            end_unix_timestamp: None,
            end_timestamp: None,
            si_storable,
            parent_id,
        };
        insert_model(db, nats, &event.id, &event).await?;

        Ok(event)
    }

    pub async fn from_si_storable(
        db: &Db,
        nats: &Connection,
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
            db,
            nats,
            message,
            payload,
            kind,
            context_list,
            parent_id,
            si_storable.billing_account_id.clone(),
            si_storable.organization_id.clone(),
            si_storable.workspace_id.clone(),
            si_storable.created_by_user_id.clone(),
        )
        .await
    }

    pub async fn node_entity_create(
        db: &Db,
        nats: &Connection,
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
            &db,
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
        db: &Db,
        nats: &Connection,
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
            &db,
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
        db: &Db,
        nats: &Connection,
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
            &db,
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
        db: &Db,
        nats: &Connection,
        change_set: &ChangeSet,
        event_parent_id: Option<String>,
    ) -> EventResult<Event> {
        let message = format!("executing changeSet {}", &change_set.name);
        let payload = serde_json::json!({
            "name": change_set.name,
        });
        let kind = EventKind::ChangeSetExecute;
        Event::from_si_storable(
            &db,
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
        db: &Db,
        nats: &Connection,
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
            &db,
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

    pub async fn running(&mut self, db: &Db, nats: &Connection) -> EventResult<()> {
        self.status = EventStatus::Running;
        upsert_model(&db, &nats, &self.id, &self).await?;
        Ok(())
    }

    pub async fn success(&mut self, db: &Db, nats: &Connection) -> EventResult<()> {
        self.status = EventStatus::Success;
        upsert_model(&db, &nats, &self.id, &self).await?;
        Ok(())
    }

    pub async fn error(&mut self, db: &Db, nats: &Connection) -> EventResult<()> {
        self.status = EventStatus::Error;
        upsert_model(&db, &nats, &self.id, &self).await?;
        Ok(())
    }

    pub async fn unknown(&mut self, db: &Db, nats: &Connection) -> EventResult<()> {
        self.status = EventStatus::Unknown;
        upsert_model(&db, &nats, &self.id, &self).await?;
        Ok(())
    }

    pub async fn log(
        &self,
        db: &Db,
        nats: &Connection,
        level: EventLogLevel,
        message: impl Into<String>,
        payload: impl Into<serde_json::Value>,
    ) -> EventResult<EventLog> {
        let message = message.into();
        let payload = payload.into();
        let log = EventLog::new(
            &db,
            &nats,
            message,
            payload,
            level,
            self.id.clone(),
            self.si_storable.billing_account_id.clone(),
            self.si_storable.organization_id.clone(),
            self.si_storable.workspace_id.clone(),
            self.si_storable.created_by_user_id.clone(),
        )
        .await?;
        Ok(log)
    }

    pub async fn has_parent(&self, db: &Db, parent_id: impl AsRef<str>) -> EventResult<bool> {
        let parent_id = parent_id.as_ref();
        let mut stack = vec![self.clone()];

        while let Some(event) = stack.pop() {
            match event.parent_id.as_ref() {
                Some(my_parent_id) => {
                    if my_parent_id == parent_id {
                        return Ok(true);
                    } else {
                        let parent: Event = get_model(
                            db,
                            my_parent_id,
                            event.si_storable.billing_account_id.clone(),
                        )
                        .await?;
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

    pub async fn save(&self, db: &Db, nats: &Connection) -> EventResult<()> {
        upsert_model(db, nats, &self.id, self).await?;
        Ok(())
    }
}
