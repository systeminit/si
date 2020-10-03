use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{delete_model, insert_model, ModelError, SiStorable, SiStorableError};

use std::time::{SystemTime, SystemTimeError};

#[derive(Error, Debug)]
pub enum EventLogError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("failure to get time: {0}")]
    Time(#[from] SystemTimeError),
}

pub type EventLogResult<T> = Result<T, EventLogError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EventLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EventLog {
    pub id: String,
    pub message: String,
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub si_storable: SiStorable,
    pub payload: serde_json::Value,
    pub level: EventLogLevel,
}

impl EventLog {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        message: impl Into<String>,
        payload: serde_json::Value,
        level: EventLogLevel,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: Option<String>,
    ) -> EventLogResult<EventLog> {
        let message = message.into();
        let si_storable = SiStorable::new(
            db,
            "eventLog",
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?;
        let id = si_storable.object_id.clone();

        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let event_log = EventLog {
            id,
            message,
            level,
            unix_timestamp,
            timestamp,
            payload,
            si_storable,
        };
        insert_model(db, nats, &event_log.id, &event_log).await?;

        Ok(event_log)
    }
}
