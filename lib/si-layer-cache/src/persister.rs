use std::sync::Arc;

use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use telemetry::prelude::*;
use tokio::{
    join,
    sync::{
        mpsc::{
            self,
        },
        oneshot,
    },
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use ulid::Ulid;

use crate::{
    db::func_run::FuncRunDb,
    error::{
        LayerDbError,
        LayerDbResult,
    },
    event::{
        LayeredEvent,
        LayeredEventClient,
        LayeredEventKind,
    },
    nats::layerdb_events_stream,
    pg::PgLayer,
};

#[derive(Debug)]
pub enum PersistMessage {
    Write((LayeredEvent, PersisterStatusWriter)),
    Evict((LayeredEvent, PersisterStatusWriter)),
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

    pub fn evict_event(&self, event: LayeredEvent) -> LayerDbResult<PersisterStatusReader> {
        let (status_write, status_read) = self.get_status_channels();
        self.tx
            .send(PersistMessage::Evict((event, status_write)))
            .map_err(Box::new)?;
        Ok(status_read)
    }
}

#[derive(Debug)]
pub struct PersisterTask {
    messages: mpsc::UnboundedReceiver<PersistMessage>,
    pg_pool: PgPool,
    layered_event_client: LayeredEventClient,
    tracker: TaskTracker,
    shutdown_token: CancellationToken,
}

impl PersisterTask {
    const NAME: &'static str = "LayerDB::PersisterTask";

    pub async fn create(
        messages: mpsc::UnboundedReceiver<PersistMessage>,
        pg_pool: PgPool,
        nats_client: &NatsClient,
        instance_id: Ulid,
        shutdown_token: CancellationToken,
    ) -> LayerDbResult<Self> {
        let tracker = TaskTracker::new();

        let context = si_data_nats::jetstream::new(nats_client.clone());
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
                        self.pg_pool.clone(),
                        self.layered_event_client.clone(),
                    );
                    self.tracker.spawn(task.write_layers(event, status_tx));
                }
                PersistMessage::Evict((event, status_tx)) => {
                    let task = PersistEventTask::new(
                        self.pg_pool.clone(),
                        self.layered_event_client.clone(),
                    );
                    self.tracker.spawn(task.evict_layers(event, status_tx));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PersisterTaskErrorKind {
    Write,
    Evict,
}

#[derive(Debug, Clone)]
pub struct PersisterTaskError {
    pub kind: PersisterTaskErrorKind,
    pub pg_error: Option<String>,
    pub nats_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PersistEventTask {
    pg_pool: PgPool,
    layered_event_client: LayeredEventClient,
}

impl PersistEventTask {
    pub fn new(pg_pool: PgPool, layered_event_client: LayeredEventClient) -> Self {
        PersistEventTask {
            pg_pool,
            layered_event_client,
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn evict_layers(self, event: LayeredEvent, status_tx: PersisterStatusWriter) {
        match self.try_evict_layers(event).await {
            Ok(_) => status_tx.send(PersistStatus::Finished),
            Err(err) => {
                error!(error = ?err, "persister evict task failed");
                status_tx.send(PersistStatus::Error(err));
            }
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn try_evict_layers(&self, event: LayeredEvent) -> LayerDbResult<()> {
        let event = Arc::new(event);

        // Write the eviction to nats
        let nats_join = self.layered_event_client.publish(event.clone()).await?;

        // Evict from to pg
        let pg_self = self.clone();
        let pg_event = event.clone();
        let pg_join = tokio::task::spawn(async move { pg_self.evict_from_pg(pg_event).await });

        match join![pg_join, nats_join] {
            (Ok(Ok(_)), Ok(Ok(_))) => Ok(()),
            (pg_res, nats_res) => {
                let kind = PersisterTaskErrorKind::Evict;
                let pg_error = match pg_res {
                    Ok(Err(e)) => Some(e.to_string()),
                    Err(e) => Some(e.to_string()),
                    _ => None,
                };
                let nats_error = match nats_res {
                    Ok(Err(e)) => Some(e.to_string()),
                    Err(e) => Some(e.to_string()),
                    _ => None,
                };
                Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                    kind,
                    pg_error,
                    nats_error,
                }))
            }
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn evict_from_pg(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        let pg_layer = PgLayer::new(self.pg_pool.clone(), event.payload.db_name.as_ref());
        pg_layer.delete(&event.payload.key).await?;
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn write_layers(self, event: LayeredEvent, status_tx: PersisterStatusWriter) {
        match self.try_write_layers(event).await {
            Ok(_) => status_tx.send(PersistStatus::Finished),
            Err(err) => {
                error!(error = ?err, "persister write task failed");
                status_tx.send(PersistStatus::Error(err));
            }
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn try_write_layers(&self, event: LayeredEvent) -> LayerDbResult<()> {
        let event = Arc::new(event);

        // Write to nats
        let nats_join = self.layered_event_client.publish(event.clone()).await?;

        // Write to pg
        let pg_self = self.clone();
        let pg_event = event.clone();
        let pg_join = tokio::task::spawn(async move { pg_self.write_to_pg(pg_event).await });

        match join![pg_join, nats_join] {
            (Ok(Ok(_)), Ok(Ok(_))) => Ok(()),
            (pg_res, nats_res) => {
                let kind = PersisterTaskErrorKind::Write;
                let pg_error = match pg_res {
                    Ok(Err(e)) => Some(e.to_string()),
                    Err(e) => Some(e.to_string()),
                    _ => None,
                };
                let nats_error = match nats_res {
                    Ok(Err(e)) => Some(e.to_string()),
                    Err(e) => Some(e.to_string()),
                    _ => None,
                };
                Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                    kind,
                    pg_error,
                    nats_error,
                }))
            }
        }
    }

    // Write an event to the pg layer
    #[instrument(level = "debug", skip_all)]
    pub async fn write_to_pg(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        let pg_layer = PgLayer::new(self.pg_pool.clone(), event.payload.db_name.as_ref());
        match event.event_kind {
            LayeredEventKind::CasInsertion
            | LayeredEventKind::ChangeBatchEvict
            | LayeredEventKind::ChangeBatchWrite
            | LayeredEventKind::EncryptedSecretInsertion
            | LayeredEventKind::Raw
            | LayeredEventKind::RebaseBatchEvict
            | LayeredEventKind::RebaseBatchWrite
            | LayeredEventKind::SnapshotEvict
            | LayeredEventKind::SnapshotWrite
            | LayeredEventKind::SplitSnapshotSubGraphEvict
            | LayeredEventKind::SplitSnapshotSubGraphWrite
            | LayeredEventKind::SplitSnapshotSuperGraphEvict
            | LayeredEventKind::SplitSnapshotSuperGraphWrite
            | LayeredEventKind::SplitRebaseBatchEvict
            | LayeredEventKind::SplitRebaseBatchWrite => {
                pg_layer
                    .insert(
                        &event.payload.key,
                        event.payload.sort_key.as_ref(),
                        &event.payload.value[..],
                    )
                    .await?;
            }
            LayeredEventKind::FuncRunLogWrite => {
                // Skip doing the write here - we don't need it. - we do it in the FunRunLog
                // write method directly, to ensure we write to PG in order.
                //
                // FuncRunLogDb::insert_to_pg(&pg_layer, &event.payload).await?
            }
            LayeredEventKind::FuncRunWrite => {
                FuncRunDb::insert_to_pg(&pg_layer, &event.payload).await?
            }
        }
        Ok(())
    }
}
