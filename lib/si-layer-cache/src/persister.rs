use std::{path::PathBuf, sync::Arc};

use si_data_nats::{async_nats::jetstream, NatsClient};
use si_data_pg::PgPool;
use telemetry::prelude::*;
use tokio::{
    join,
    sync::{
        mpsc::{self},
        oneshot,
    },
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::{
    disk_cache::DiskCache,
    error::{LayerDbError, LayerDbResult},
    event::{LayeredEvent, LayeredEventClient},
    nats::layerdb_events_stream,
    pg::PgLayer,
};

#[derive(Debug)]
pub enum PersistMessage {
    Write((LayeredEvent, PersisterStatusWriter)),
}

#[derive(Debug)]
pub enum PersistStatus {
    Finished,
    Error(LayerDbError),
}

#[derive(Debug)]
pub struct PersisterStatusWriter {
    tx: oneshot::Sender<PersistStatus>,
}

impl PersisterStatusWriter {
    pub fn new(tx: oneshot::Sender<PersistStatus>) -> Self {
        Self { tx }
    }

    pub fn send(self, msg: PersistStatus) {
        // If the other end isn't listening, we really don't care!
        let _ = self.tx.send(msg);
    }
}

#[derive(Debug)]
pub struct PersisterStatusReader {
    rx: oneshot::Receiver<PersistStatus>,
}

impl PersisterStatusReader {
    pub fn new(rx: oneshot::Receiver<PersistStatus>) -> Self {
        Self { rx }
    }

    pub async fn get_status(self) -> LayerDbResult<PersistStatus> {
        Ok(self.rx.await?)
    }
}

#[derive(Debug, Clone)]
pub struct PersisterClient {
    tx: mpsc::UnboundedSender<PersistMessage>,
}

impl PersisterClient {
    pub fn new(tx: mpsc::UnboundedSender<PersistMessage>) -> PersisterClient {
        PersisterClient { tx }
    }

    fn get_status_channels(&self) -> (PersisterStatusWriter, PersisterStatusReader) {
        let (status_tx, status_rx) = oneshot::channel();
        (
            PersisterStatusWriter::new(status_tx),
            PersisterStatusReader::new(status_rx),
        )
    }

    pub fn write_event(&self, event: LayeredEvent) -> LayerDbResult<PersisterStatusReader> {
        let (status_write, status_read) = self.get_status_channels();
        self.tx
            .send(PersistMessage::Write((event, status_write)))
            .map_err(Box::new)?;
        Ok(status_read)
    }
}

#[derive(Debug)]
pub struct PersisterTask {
    messages: mpsc::UnboundedReceiver<PersistMessage>,
    disk_path: PathBuf,
    pg_pool: PgPool,
    layered_event_client: LayeredEventClient,
    tracker: TaskTracker,
    shutdown_token: CancellationToken,
}

impl PersisterTask {
    const NAME: &'static str = "LayerDB::PersisterTask";

    pub async fn create(
        messages: mpsc::UnboundedReceiver<PersistMessage>,
        disk_path: PathBuf,
        pg_pool: PgPool,
        nats_client: &NatsClient,
        instance_id: Ulid,
        shutdown_token: CancellationToken,
    ) -> LayerDbResult<Self> {
        let tracker = TaskTracker::new();

        let context = jetstream::new(nats_client.as_inner().clone());
        // Ensure the Jetstream is created
        let _stream =
            layerdb_events_stream(&context, nats_client.metadata().subject_prefix()).await?;

        let layered_event_client = LayeredEventClient::new(
            nats_client
                .metadata()
                .subject_prefix()
                .map(|s| s.to_owned()),
            instance_id,
            context.clone(),
        );

        Ok(Self {
            messages,
            disk_path,
            pg_pool,
            layered_event_client,
            tracker,
            shutdown_token,
        })
    }

    pub async fn run(mut self) {
        let shutdown_token = self.shutdown_token.clone();

        loop {
            tokio::select! {
                _ = self.process_messages() => {
                    // When no messages remain, channel is fully drained and we are done
                    break;
                }
                _ = shutdown_token.cancelled() => {
                    debug!(task = Self::NAME, "received cancellation");
                    // Close receiver channel to ensure to further values can be received and
                    // continue to process remaining values until channel is fully drained
                    self.messages.close();
                }
            }
        }

        // All remaining work has been dispatched (i.e. spawned) so no more tasks will be spawned
        self.tracker.close();
        // Wait for all in-flight writes work to complete
        self.tracker.wait().await;

        debug!(task = Self::NAME, "shutdown complete");
    }

    async fn process_messages(&mut self) {
        while let Some(msg) = self.messages.recv().await {
            match msg {
                PersistMessage::Write((event, status_tx)) => {
                    let task = PersistEventTask::new(
                        self.disk_path.clone(),
                        self.pg_pool.clone(),
                        self.layered_event_client.clone(),
                    );
                    self.tracker.spawn(task.write_layers(event, status_tx));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PersisterTaskError {
    pub disk_error: Option<String>,
    pub pg_error: Option<String>,
    pub nats_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PersistEventTask {
    disk_path: PathBuf,
    pg_pool: PgPool,
    layered_event_client: LayeredEventClient,
}

impl PersistEventTask {
    pub fn new(
        disk_path: PathBuf,
        pg_pool: PgPool,
        layered_event_client: LayeredEventClient,
    ) -> Self {
        PersistEventTask {
            disk_path,
            pg_pool,
            layered_event_client,
        }
    }

    pub async fn write_layers(self, event: LayeredEvent, status_tx: PersisterStatusWriter) {
        match self.try_write_layers(event).await {
            Ok(_) => status_tx.send(PersistStatus::Finished),
            Err(e) => {
                println!("wtf: {:?}", e);
                status_tx.send(PersistStatus::Error(e));
            }
        }
    }

    pub async fn try_write_layers(&self, event: LayeredEvent) -> LayerDbResult<()> {
        let event = Arc::new(event);

        // Write to nats
        let nats_join = self.layered_event_client.publish(event.clone()).await?;

        // Write to pg
        let pg_self = self.clone();
        let pg_event = event.clone();
        let pg_join = tokio::task::spawn(async move { pg_self.write_to_pg(pg_event).await });

        // Write to disk cache
        let disk_cache = DiskCache::new(&self.disk_path, event.payload.db_name.to_string())?;
        let disk_join = tokio::task::spawn(async move { disk_cache.write_to_disk(event).await });

        match join![disk_join, pg_join, nats_join] {
            (Ok(Ok(_)), Ok(Ok(_)), Ok(Ok(_))) => Ok(()),
            (disk_res, Ok(pg_res), Ok(nats_res)) => {
                Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                    disk_error: disk_res.err().map(|e| e.to_string()),
                    pg_error: pg_res.err().map(|e| e.to_string()),
                    nats_error: nats_res.err().map(|e| e.to_string()),
                }))
            }
            (Ok(_), Ok(_), Err(e)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: None,
                pg_error: None,
                nats_error: Some(e.to_string()),
            })),
            (Ok(_), Err(e), Ok(_)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: None,
                pg_error: Some(e.to_string()),
                nats_error: None,
            })),
            (Ok(_), Err(p), Err(n)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: None,
                pg_error: Some(p.to_string()),
                nats_error: Some(n.to_string()),
            })),
            (Err(d), Ok(_), Err(n)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: Some(d.to_string()),
                pg_error: None,
                nats_error: Some(n.to_string()),
            })),
            (Err(d), Err(p), Ok(_)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: Some(d.to_string()),
                pg_error: Some(p.to_string()),
                nats_error: None,
            })),
            (Err(d), Err(p), Err(n)) => {
                Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                    disk_error: Some(d.to_string()),
                    pg_error: Some(p.to_string()),
                    nats_error: Some(n.to_string()),
                }))
            }
        }
    }

    // Write an event to the pg layer
    pub async fn write_to_pg(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        let pg_layer = PgLayer::new(self.pg_pool.clone(), event.payload.db_name.as_ref());
        pg_layer
            .insert(
                &event.payload.key,
                event.payload.sort_key.as_ref(),
                &event.payload.value[..],
            )
            .await?;
        Ok(())
    }
}
