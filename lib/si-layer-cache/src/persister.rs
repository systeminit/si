use std::sync::Arc;

use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use tokio::{
    join,
    sync::{mpsc, oneshot},
};

use crate::{
    error::{LayerDbError, LayerDbResult},
    event::LayeredEvent,
    pg::PgLayer,
};

#[derive(Debug)]
pub enum PersistMessage {
    Write((LayeredEvent, PersisterStatusWriter)),
    Shutdown,
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

#[derive(Debug, Clone)]
pub struct PersisterServer {
    pub sled: sled::Db,
    pub pg_pool: PgPool,
    pub nats_client: NatsClient,
}

impl PersisterServer {
    pub async fn start(
        mut rx: mpsc::UnboundedReceiver<PersistMessage>,
        sled: sled::Db,
        pg_pool: PgPool,
        nats_client: NatsClient,
    ) {
        let server = PersisterServer {
            sled,
            pg_pool,
            nats_client,
        };

        while let Some(msg) = rx.recv().await {
            match msg {
                PersistMessage::Write((event, status_tx)) => {
                    let task = PersisterTask::new(
                        server.sled.clone(),
                        server.pg_pool.clone(),
                        server.nats_client.clone(),
                    );
                    tokio::spawn(async move { task.write_layers(event, status_tx).await });
                }
                PersistMessage::Shutdown => break,
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
pub struct PersisterTask {
    sled: sled::Db,
    pg_pool: PgPool,
    _nats_client: NatsClient,
}

impl PersisterTask {
    pub fn new(sled: sled::Db, pg_pool: PgPool, nats_client: NatsClient) -> Self {
        PersisterTask {
            sled,
            pg_pool,
            _nats_client: nats_client,
        }
    }

    pub async fn write_layers(&self, event: LayeredEvent, status_tx: PersisterStatusWriter) {
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
        let disk_join = tokio::task::spawn_blocking(move || disk_self.write_to_disk(disk_event));

        // Write to nats - TODO for adam and fletcher

        // Write to pg
        let pg_self = self.clone();
        let pg_event = event.clone();
        let pg_join = tokio::task::spawn(async move { pg_self.write_to_pg(pg_event).await });

        match join![disk_join, pg_join] {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(e)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: None,
                nats_error: None,
                pg_error: Some(e.to_string()),
            })),
            (Err(e), Ok(_)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: Some(e.to_string()),
                nats_error: None,
                pg_error: None,
            })),
            (Err(d), Err(p)) => Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                disk_error: Some(d.to_string()),
                nats_error: None,
                pg_error: Some(p.to_string()),
            })),
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
                &event.payload.key[..],
                event.payload.sort_key.as_ref(),
                &event.payload.value[..],
            )
            .await?;
        Ok(())
    }
}
