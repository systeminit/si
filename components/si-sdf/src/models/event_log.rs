use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::pin::Pin;
use strum_macros::Display;
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::data::{EventLogFS, EventLogFSError, NatsConn, NatsTxnError, PgPool, PgTxn};
use crate::models::{
    list_model, next_update_clock, Event, EventResult, ListReply, ModelError, OrderByDirection,
    OutputLine, OutputLineStream, PageToken, Query, SiStorable, UpdateClockError,
};

#[derive(Error, Debug)]
pub enum EventLogError {
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("eventLogFS error: {0}")]
    EventLogFS(#[from] EventLogFSError),
    #[error("io error: {0}")]
    IO(#[from] tokio::io::Error),
}

pub type EventLogResult<T> = Result<T, EventLogError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Display, Clone)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EventLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventLog {
    pub id: String,
    pub message: String,
    pub payload: serde_json::Value,
    pub level: EventLogLevel,
    pub event_id: String,
    pub has_output_line: bool,
    pub unix_timestamp: i64,
    pub timestamp: String,
    pub si_storable: SiStorable,

    #[serde(skip)]
    streams: Streams,
}

#[derive(Default)]
struct Streams {
    stdout: Option<Pin<Box<dyn AsyncWrite + Sync + Send>>>,
    stderr: Option<Pin<Box<dyn AsyncWrite + Sync + Send>>>,
    all: Option<Pin<Box<dyn AsyncWrite + Sync + Send>>>,
}

impl PartialEq for Streams {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for Streams {}

impl fmt::Debug for Streams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Streams").finish()
    }
}

impl Clone for Streams {
    fn clone(&self) -> Self {
        Self {
            stdout: None,
            stderr: None,
            all: None,
        }
    }
}

impl EventLog {
    pub async fn new(
        pg: &PgPool,
        nats_conn: &NatsConn,
        message: impl Into<String>,
        payload: serde_json::Value,
        level: EventLogLevel,
        event_id: String,
        workspace_id: String,
    ) -> EventLogResult<EventLog> {
        let message = message.into();
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let update_clock = next_update_clock(&workspace_id).await?;

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let row = txn
            .query_one(
                "SELECT object FROM event_log_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &message,
                    &payload,
                    &event_id,
                    &level.to_string(),
                    &timestamp,
                    &unix_timestamp,
                    &workspace_id,
                    &update_clock.epoch,
                    &update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;

        txn.commit().await?;
        nats.commit().await?;

        let object: EventLog = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn has_parent(
        &self,
        txn: &PgTxn<'_>,
        parent_id: impl AsRef<str>,
    ) -> EventResult<bool> {
        let parent_id = parent_id.as_ref();
        if self.event_id == parent_id {
            return Ok(true);
        }
        let event: Event = Event::get(&txn, &self.event_id).await?;
        event.has_parent(&txn, parent_id).await
    }

    pub async fn output_line(
        &mut self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        event_log_fs: &EventLogFS,
        stream: OutputLineStream,
        line: impl Into<String>,
        closed: bool,
    ) -> EventLogResult<OutputLine> {
        if !self.has_output_line {
            self.has_output_line = true;
            self.save(pg, nats_conn).await?;
        }

        let mut si_storable = self.si_storable.clone();
        si_storable.type_name = "outputLine".to_string();

        let output_line = OutputLine::new(
            line,
            stream.clone(),
            self.event_id.clone(),
            self.id.clone(),
            closed,
            si_storable,
        );

        let nats = nats_conn.transaction();

        self.write_line(event_log_fs, &output_line).await?;
        nats.publish(&output_line).await?;

        nats.commit().await?;

        Ok(output_line)
    }

    pub async fn get(txn: &PgTxn<'_>, event_log_id: impl AsRef<str>) -> EventLogResult<EventLog> {
        let id = event_log_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM event_log_get_v1($1)", &[&id])
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
            "event_logs",
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

    pub async fn save(&mut self, pg: &PgPool, nats_conn: &NatsConn) -> EventLogResult<()> {
        let update_clock = next_update_clock(&self.si_storable.workspace_id).await?;
        self.si_storable.update_clock = update_clock;

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();
        let json = serde_json::to_value(&self)?;

        let row = txn
            .query_one("SELECT object FROM event_log_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;

        txn.commit().await?;
        nats.commit().await?;

        let updated: Self = serde_json::from_value(updated_result)?;

        // We're not mem::swap'ing here because we want to hold onto the `streams` and right now
        // `Streams` don't correctly clone itself (on purpose)

        self.message = updated.message;
        self.payload = updated.payload;
        self.level = updated.level;
        self.event_id = updated.event_id;
        self.has_output_line = updated.has_output_line;
        self.unix_timestamp = updated.unix_timestamp;
        self.timestamp = updated.timestamp;
        self.si_storable = updated.si_storable;

        Ok(())
    }

    async fn write_line(
        &mut self,
        event_log_fs: &EventLogFS,
        output_line: &OutputLine,
    ) -> EventLogResult<()> {
        match output_line.stream {
            OutputLineStream::Stdout => {
                Self::write_line_to_stream(
                    &self.id,
                    event_log_fs,
                    output_line,
                    &mut self.streams.stdout,
                )
                .await
            }
            OutputLineStream::Stderr => {
                Self::write_line_to_stream(
                    &self.id,
                    event_log_fs,
                    output_line,
                    &mut self.streams.stderr,
                )
                .await
            }
            OutputLineStream::All => {
                Self::write_line_to_stream(
                    &self.id,
                    event_log_fs,
                    output_line,
                    &mut self.streams.all,
                )
                .await
            }
        }
    }

    async fn write_line_to_stream(
        id: &str,
        event_log_fs: &EventLogFS,
        output_line: &OutputLine,
        write_handle: &mut Option<Pin<Box<dyn AsyncWrite + Sync + Send>>>,
    ) -> EventLogResult<()> {
        // If the output line has a closed marker, then we drop the writable file handle,
        // finalize the EventLogFS, and early return
        if output_line.closed {
            {
                let handle = write_handle.take();
                drop(handle);
            }
            event_log_fs.finalize(id, &output_line.stream).await?;
            return Ok(());
        }

        // If this is the first write to the stream, get a writable file handle from
        // EventLogFS and save it
        if write_handle.is_none() {
            *write_handle = Some(
                event_log_fs
                    .get_write_handle(id, &output_line.stream)
                    .await?,
            );
        }

        // Write the output line to the stream
        write_handle
            .as_mut()
            .unwrap()
            .write_all(output_line.line.as_bytes())
            .await?;
        write_handle
            .as_mut()
            .unwrap()
            .write_all("\n".as_bytes())
            .await?;
        write_handle.as_mut().unwrap().flush().await?;

        Ok(())
    }
}
