use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{
    insert_model, upsert_model, ModelError, OutputLine, OutputLineError, OutputLineStream,
    SiStorable, SiStorableError,
};

#[derive(Error, Debug)]
pub enum EventLogError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("outputLine error: {0}")]
    OutputLine(#[from] OutputLineError),
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
    Fatal,
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
    pub event_id: String,
    pub has_output_line: bool,
}

impl EventLog {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        message: impl Into<String>,
        payload: serde_json::Value,
        level: EventLogLevel,
        event_id: String,
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

        let has_output_line = false;

        let event_log = EventLog {
            id,
            message,
            level,
            unix_timestamp,
            timestamp,
            payload,
            event_id,
            si_storable,
            has_output_line,
        };
        insert_model(db, nats, &event_log.id, &event_log).await?;

        Ok(event_log)
    }

    pub async fn output_line(
        &mut self,
        db: &Db,
        nats: &Connection,
        stream: OutputLineStream,
        line: impl Into<String>,
    ) -> EventLogResult<OutputLine> {

        if !self.has_output_line {
            self.has_output_line = true;
            self.save(db, nats).await?;
        }

        let output_line = OutputLine::new(
            db,
            nats,
            line,
            stream,
            self.event_id.clone(),
            self.id.clone(),
            self.si_storable.billing_account_id.clone(),
            self.si_storable.organization_id.clone(),
            self.si_storable.workspace_id.clone(),
            self.si_storable.created_by_user_id.clone(),
        )
        .await?;
        Ok(output_line)
    }

    pub async fn save(&self, db: &Db, nats: &Connection) -> EventLogResult<()> {
        upsert_model(db, nats, &self.id, self).await?;
        Ok(())
    }
}
