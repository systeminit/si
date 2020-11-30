use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{Connection, Db};
use crate::models::{
    get_model, insert_model, upsert_model, Event, EventResult, ModelError, SiStorable,
    SiStorableError,
};

#[derive(Error, Debug)]
pub enum OutputLineError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type OutputLineResult<T> = Result<T, OutputLineError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputLineStream {
    Stdout,
    Stderr,
    All,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutputLine {
    pub id: String,
    pub line: String,
    pub stream: OutputLineStream,
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub si_storable: SiStorable,
    pub event_id: String,
    pub event_log_id: String,
}

impl OutputLine {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        line: impl Into<String>,
        stream: OutputLineStream,
        event_id: String,
        event_log_id: String,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: Option<String>,
    ) -> OutputLineResult<OutputLine> {
        let line = line.into();
        let si_storable = SiStorable::new(
            db,
            "outputLine",
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

        let output_line = OutputLine {
            id,
            line,
            stream,
            unix_timestamp,
            timestamp,
            event_id,
            event_log_id,
            si_storable,
        };
        insert_model(db, nats, &output_line.id, &output_line).await?;

        Ok(output_line)
    }

    pub async fn has_parent(&self, db: &Db, parent_id: impl AsRef<str>) -> EventResult<bool> {
        let parent_id = parent_id.as_ref();
        if self.event_id == parent_id {
            return Ok(true);
        }
        let event: Event =
            get_model(db, &self.event_id, &self.si_storable.billing_account_id).await?;
        event.has_parent(&db, parent_id).await
    }

    pub async fn save(&self, db: &Db, nats: &Connection) -> OutputLineResult<()> {
        upsert_model(db, nats, &self.id, self).await?;
        Ok(())
    }
}
