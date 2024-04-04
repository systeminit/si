use std::sync::Arc;

use si_data_nats::{async_nats::jetstream, HeaderMap, NatsClient};
use si_data_pg::PgPool;
use telemetry::prelude::*;
use tokio::{
    join,
    sync::{mpsc, oneshot},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

use crate::{
    chunking_nats::ChunkingNats,
    error::{LayerDbError, LayerDbResult},
    event::LayeredEvent,
    nats::{
        layerdb_events_stream, subject, NATS_HEADER_DB_NAME, NATS_HEADER_INSTANCE_ID,
        NATS_HEADER_KEY,
    },
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
        self.tx.send(PersistMessage::Write((event, status_write)))?;
        Ok(status_read)
    }
}

#[derive(Debug)]
pub struct PersisterTask {
    messages: mpsc::UnboundedReceiver<PersistMessage>,
    sled: sled::Db,
    pg_pool: PgPool,
    nats: ChunkingNats,
    instance_id: Ulid,
    tracker: TaskTracker,
    shutdown_token: CancellationToken,
}

impl PersisterTask {
    const NAME: &'static str = "LayerDB::PersisterTask";

    pub async fn create(
        messages: mpsc::UnboundedReceiver<PersistMessage>,
        sled: sled::Db,
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
        let nats = ChunkingNats::new(
            nats_client
                .metadata()
                .subject_prefix()
                .map(|s| s.to_owned()),
            context,
        );

        Ok(Self {
            messages,
            sled,
            pg_pool,
            nats,
            instance_id,
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
                        self.sled.clone(),
                        self.pg_pool.clone(),
                        self.nats.clone(),
                        self.instance_id,
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
    sled: sled::Db,
    pg_pool: PgPool,
    nats: ChunkingNats,
    instance_id: Ulid,
}

impl PersistEventTask {
    pub fn new(sled: sled::Db, pg_pool: PgPool, nats: ChunkingNats, instance_id: Ulid) -> Self {
        PersistEventTask {
            sled,
            pg_pool,
            nats,
            instance_id,
        }
    }

    pub async fn write_layers(self, event: LayeredEvent, status_tx: PersisterStatusWriter) {
        match self.try_write_layers(event).await {
            Ok(_) => status_tx.send(PersistStatus::Finished),
            Err(e) => status_tx.send(PersistStatus::Error(e)),
        }
    }

    pub async fn try_write_layers(&self, event: LayeredEvent) -> LayerDbResult<()> {
        let event = Arc::new(event);

        // Write to disk cache
        let disk_self = self.clone();
        let disk_event = event.clone();
        let disk_result = disk_self.write_to_disk(disk_event);

        // Write to nats
        let nats_join = if event.gossip {
            let nats_subject = subject::for_event(self.nats.prefix(), event.as_ref());
            let nats = self.nats.clone();
            let mut nats_headers = HeaderMap::new();
            nats_headers.insert(NATS_HEADER_DB_NAME, event.payload.db_name.as_str());
            nats_headers.insert(NATS_HEADER_KEY, event.payload.key.as_ref());
            nats_headers.insert(
                NATS_HEADER_INSTANCE_ID,
                self.instance_id.to_string().as_str(),
            );
            let nats_payload = postcard::to_stdvec(&event)?;
            tokio::spawn(async move {
                nats.publish_with_headers(nats_subject, nats_headers, nats_payload.into())
                    .await
            })
        } else {
            tokio::spawn(async move { Ok(()) })
        };

        // Write to pg
        let pg_self = self.clone();
        let pg_event = event.clone();
        let pg_join = tokio::task::spawn(async move { pg_self.write_to_pg(pg_event).await });

        let join_results = join![pg_join, nats_join];
        match (disk_result, join_results.0, join_results.1) {
            (Ok(_), Ok(_), Ok(_)) => Ok(()),
            (Err(e), Ok(_), Ok(_)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: Some(e.to_string()),
                pg_error: None,
                nats_error: None,
            })),
            (Ok(_), Err(e), Ok(_)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: None,
                pg_error: Some(e.to_string()),
                nats_error: None,
            })),
            (Ok(_), Ok(_), Err(e)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: None,
                pg_error: None,
                nats_error: Some(e.to_string()),
            })),
            (Err(d), Err(p), Ok(_)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: Some(d.to_string()),
                pg_error: Some(p.to_string()),
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
            (Err(d), Err(p), Err(n)) => {
                Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                    disk_error: Some(d.to_string()),
                    pg_error: Some(p.to_string()),
                    nats_error: Some(n.to_string()),
                }))
            }
        }
    }

    pub fn write_to_disk(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        let tree = self.sled.open_tree(event.payload.db_name.as_ref())?;
        tree.insert(&*event.payload.key, &event.payload.value[..])?;
        Ok(())
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
