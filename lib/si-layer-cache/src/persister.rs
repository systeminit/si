use std::{
    path::PathBuf,
    sync::Arc,
};

use chrono::Utc;
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use telemetry::prelude::*;
use telemetry_utils::metric;
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
    BackendType,
    db::{
        cas,
        change_batch,
        encrypted_secret,
        func_run::{
            self,
            FuncRunDb,
        },
        func_run_log,
        rebase_batch,
        split_snapshot_rebase_batch,
        split_snapshot_subgraph,
        split_snapshot_supergraph,
        workspace_snapshot,
    },
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
    EvictMemoryOnly((LayeredEvent, PersisterStatusWriter)),
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

    pub fn evict_memory_only_event(
        &self,
        event: LayeredEvent,
    ) -> LayerDbResult<PersisterStatusReader> {
        let (status_write, status_read) = self.get_status_channels();
        self.tx
            .send(PersistMessage::EvictMemoryOnly((event, status_write)))
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
    retry_queue_command_tx: mpsc::UnboundedSender<crate::retry_queue::RetryQueueMessage>,
    pending_retry_rx:
        mpsc::UnboundedReceiver<(crate::event::LayeredEvent, crate::retry_queue::RetryHandle)>,
}

impl PersisterTask {
    const NAME: &'static str = "LayerDB::PersisterTask";

    pub async fn create(
        messages: mpsc::UnboundedReceiver<PersistMessage>,
        pg_pool: PgPool,
        nats_client: &NatsClient,
        instance_id: Ulid,
        retry_queue_base_path: PathBuf,
        shutdown_token: CancellationToken,
    ) -> LayerDbResult<Self> {
        use crate::retry_queue::{
            RetryQueueConfig,
            RetryQueueManager,
        };

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

        // Create channels for RetryQueueManager communication
        let (retry_queue_command_tx, retry_queue_command_rx) = mpsc::unbounded_channel();
        let (pending_retry_tx, pending_retry_rx) = mpsc::unbounded_channel();

        // Initialize retry queue manager with provided path
        let retry_queue_config = RetryQueueConfig {
            base_path: retry_queue_base_path,
            ..Default::default()
        };

        let mut retry_queue_manager = RetryQueueManager::new(retry_queue_config);

        // Scan for existing retry queues on startup
        const CACHE_NAMES: &[&str] = &[
            cas::CACHE_NAME,
            change_batch::CACHE_NAME,
            encrypted_secret::CACHE_NAME,
            func_run::CACHE_NAME,
            func_run_log::CACHE_NAME,
            rebase_batch::CACHE_NAME,
            workspace_snapshot::CACHE_NAME,
            split_snapshot_subgraph::CACHE_NAME,
            split_snapshot_supergraph::CACHE_NAME,
            split_snapshot_rebase_batch::CACHE_NAME,
        ];
        retry_queue_manager
            .scan_existing_queues(CACHE_NAMES)
            .await?;

        // Spawn RetryQueueManager as independent task
        tracker.spawn(retry_queue_manager.run(
            retry_queue_command_rx,
            pending_retry_tx,
            shutdown_token.clone(),
        ));

        Ok(Self {
            messages,
            pg_pool,
            layered_event_client,
            tracker,
            shutdown_token,
            retry_queue_command_tx,
            pending_retry_rx,
        })
    }

    pub async fn run(mut self) {
        let shutdown_token = self.shutdown_token.clone();

        loop {
            tokio::select! {
                biased;

                // Priority 1: Shutdown signal (highest)
                _ = shutdown_token.cancelled() => {
                    debug!(task = Self::NAME, "received cancellation");
                    // Close receiver channel to ensure no further values can be received
                    self.messages.close();
                    break;
                }

                // Priority 2: New messages from channel
                Some(msg) = self.messages.recv() => {
                    self.spawn_persist_task(msg);
                }

                // Priority 3: Ready retries from RetryQueueManager
                Some((event, handle)) = self.pending_retry_rx.recv() => {
                    self.spawn_retry_task(event, handle);
                }
            }
        }

        // Drain remaining messages but don't process retry queue during shutdown
        while let Some(msg) = self.messages.recv().await {
            self.spawn_persist_task(msg);
        }

        // All remaining work has been dispatched (i.e. spawned) so no more tasks will be spawned
        self.tracker.close();
        // Wait for all in-flight writes work to complete
        self.tracker.wait().await;

        debug!(task = Self::NAME, "shutdown complete");
    }

    fn spawn_persist_task(&mut self, msg: PersistMessage) {
        match msg {
            PersistMessage::Write((event, status_tx)) => {
                let task =
                    PersistEventTask::new(self.pg_pool.clone(), self.layered_event_client.clone());
                let retry_queue_command_tx = self.retry_queue_command_tx.clone();
                let cache_name = event.payload.db_name.to_string();

                metric!(
                    counter.layer_cache_persister_write_attempted = 1,
                    cache_name = &cache_name,
                    backend = BackendType::Postgres.as_ref(),
                    event_kind = event.event_kind.as_ref()
                );

                self.tracker.spawn(async move {
                    match task.try_write_layers(event.clone(), false).await {
                        Ok(_) => {
                            metric!(
                                counter.layer_cache_persister_write_success = 1,
                                cache_name = &cache_name,
                                backend = BackendType::Postgres.as_ref(),
                                event_kind = event.event_kind.as_ref()
                            );

                            // Emit end-to-end persistence latency
                            let latency = Utc::now()
                                .signed_duration_since(event.metadata.timestamp)
                                .to_std()
                                .unwrap_or_default();
                            metric!(
                                histogram.layer_cache_persistence_latency_seconds = latency.as_secs_f64(),
                                cache_name = &cache_name,
                                backend = BackendType::Postgres.as_ref(),
                                operation = "write",
                                event_kind = event.event_kind.as_ref()
                            );

                            status_tx.send(PersistStatus::Finished)
                        }
                        Err(err) => {
                            // Check if error is retryable and enqueue if so
                            if crate::retry_queue::is_retryable_error(&err) {
                                metric!(
                                    counter.layer_cache_persister_write_failed_retryable = 1,
                                    cache_name = &cache_name,
                                    backend = BackendType::Postgres.as_ref(),
                                    event_kind = event.event_kind.as_ref()
                                );
                                let _ = retry_queue_command_tx
                                    .send(crate::retry_queue::RetryQueueMessage::Enqueue(event));
                            } else {
                                metric!(
                                    counter.layer_cache_persister_write_failed_permanent = 1,
                                    cache_name = &cache_name,
                                    backend = BackendType::Postgres.as_ref(),
                                    event_kind = event.event_kind.as_ref()
                                );
                                error!(error = ?err, "persister write task failed with non-retryable error");
                            }
                            status_tx.send(PersistStatus::Error(err));
                        }
                    }
                });
            }
            PersistMessage::Evict((event, status_tx)) => {
                let task =
                    PersistEventTask::new(self.pg_pool.clone(), self.layered_event_client.clone());
                let retry_queue_command_tx = self.retry_queue_command_tx.clone();
                let cache_name = event.payload.db_name.to_string();

                metric!(
                    counter.layer_cache_persister_evict_attempted = 1,
                    cache_name = &cache_name
                );

                self.tracker.spawn(async move {
                    match task.try_evict_layers(event.clone()).await {
                        Ok(_) => {
                            metric!(
                                counter.layer_cache_persister_evict_success = 1,
                                cache_name = &cache_name
                            );
                            status_tx.send(PersistStatus::Finished)
                        }
                        Err(err) => {
                            // Check if error is retryable and enqueue if so
                            if crate::retry_queue::is_retryable_error(&err) {
                                metric!(
                                    counter.layer_cache_persister_evict_failed_retryable = 1,
                                    cache_name = &cache_name
                                );
                                let _ = retry_queue_command_tx
                                    .send(crate::retry_queue::RetryQueueMessage::Enqueue(event));
                            } else {
                                metric!(
                                    counter.layer_cache_persister_evict_failed_permanent = 1,
                                    cache_name = &cache_name
                                );
                                error!(error = ?err, "persister evict task failed with non-retryable error");
                            }
                            status_tx.send(PersistStatus::Error(err));
                        }
                    }
                });
            }
            PersistMessage::EvictMemoryOnly((event, status_tx)) => {
                let task =
                    PersistEventTask::new(self.pg_pool.clone(), self.layered_event_client.clone());
                let retry_queue_command_tx = self.retry_queue_command_tx.clone();
                let cache_name = event.payload.db_name.to_string();

                metric!(
                    counter.layer_cache_persister_evict_memory_only_attempted = 1,
                    cache_name = &cache_name
                );

                self.tracker.spawn(async move {
                    match task.try_evict_memory_only(event.clone()).await {
                        Ok(_) => {
                            metric!(
                                counter.layer_cache_persister_evict_memory_only_success = 1,
                                cache_name = &cache_name
                            );
                            status_tx.send(PersistStatus::Finished)
                        }
                        Err(err) => {
                            // Check if error is retryable and enqueue if so
                            if crate::retry_queue::is_retryable_error(&err) {
                                metric!(
                                    counter.layer_cache_persister_evict_memory_only_failed_retryable = 1,
                                    cache_name = &cache_name
                                );
                                let _ = retry_queue_command_tx
                                    .send(crate::retry_queue::RetryQueueMessage::Enqueue(event));
                            } else {
                                metric!(
                                    counter.layer_cache_persister_evict_memory_only_failed_permanent = 1,
                                    cache_name = &cache_name
                                );
                                error!(error = ?err, "persister evict memory only task failed with non-retryable error");
                            }
                            status_tx.send(PersistStatus::Error(err));
                        }
                    }
                });
            }
        }
    }

    fn spawn_retry_task(&mut self, event: LayeredEvent, handle: crate::retry_queue::RetryHandle) {
        let task = PersistEventTask::new(self.pg_pool.clone(), self.layered_event_client.clone());
        let retry_queue_command_tx = self.retry_queue_command_tx.clone();
        let cache_name = handle.cache_name.clone();

        metric!(
            counter.layer_cache_persister_retry_attempted = 1,
            cache_name = &cache_name
        );

        self.tracker.spawn(async move {
            let start = std::time::Instant::now();

            // Attempt the retry
            let result = task.try_write_layers(event, true).await;

            let duration = start.elapsed().as_secs_f64();
            metric!(
                histogram.layer_cache_persister_retry_duration_seconds = duration,
                cache_name = &cache_name
            );

            match result {
                Ok(_) => {
                    // Success - tell manager to remove from queue
                    let _ = retry_queue_command_tx
                        .send(crate::retry_queue::RetryQueueMessage::MarkSuccess(handle));
                }
                Err(err) if crate::retry_queue::is_retryable_error(&err) => {
                    // Retryable failure - update backoff
                    let _ = retry_queue_command_tx.send(
                        crate::retry_queue::RetryQueueMessage::MarkRetryableFailure(handle, err),
                    );
                }
                Err(err) => {
                    // Permanent failure - remove from queue
                    error!(
                        cache.name = %cache_name,
                        error = ?err,
                        "retry failed with non-retryable error"
                    );
                    let _ = retry_queue_command_tx.send(
                        crate::retry_queue::RetryQueueMessage::MarkPermanentFailure(handle),
                    );
                }
            }
        });
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
        let start = std::time::Instant::now();
        let cache_name = event.payload.db_name.to_string();
        let event = Arc::new(event);

        // Write the eviction to nats
        let nats_join = self.layered_event_client.publish(event.clone()).await?;

        // Evict from to pg
        let pg_self = self.clone();
        let pg_event = event.clone();
        let pg_join = tokio::task::spawn(async move { pg_self.evict_from_pg(pg_event).await });

        let result = match join![pg_join, nats_join] {
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

                // Track which component failed
                if pg_error.is_some() && nats_error.is_some() {
                    info!(
                        metrics = true,
                        counter.layer_cache_persister_both_error = 1,
                        cache_name = &cache_name,
                        operation = "evict"
                    );
                } else if pg_error.is_some() {
                    info!(
                        metrics = true,
                        counter.layer_cache_persister_pg_error = 1,
                        cache_name = &cache_name,
                        operation = "evict"
                    );
                } else if nats_error.is_some() {
                    info!(
                        metrics = true,
                        counter.layer_cache_persister_nats_error = 1,
                        cache_name = &cache_name,
                        operation = "evict"
                    );
                }

                Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                    kind,
                    pg_error,
                    nats_error,
                }))
            }
        };

        let duration = start.elapsed().as_secs_f64();
        let status = if result.is_ok() { "success" } else { "error" };
        info!(
            metrics = true,
            histogram.layer_cache_persister_evict_duration_seconds = duration,
            cache_name = &cache_name,
            status = status
        );

        result
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn try_evict_memory_only(&self, event: LayeredEvent) -> LayerDbResult<()> {
        let start = std::time::Instant::now();
        let cache_name = event.payload.db_name.to_string();
        let event = Arc::new(event);

        // Only publish to NATS, skip PostgreSQL deletion
        let nats_join = self.layered_event_client.publish(event.clone()).await?;
        let result = nats_join.await.map_err(|e| {
            info!(
                metrics = true,
                counter.layer_cache_persister_nats_error = 1,
                cache_name = &cache_name,
                operation = "evict_memory_only"
            );
            LayerDbError::PersisterTaskFailed(PersisterTaskError {
                kind: PersisterTaskErrorKind::Evict,
                pg_error: None,
                nats_error: Some(e.to_string()),
            })
        })?;

        let duration = start.elapsed().as_secs_f64();
        let status = if result.is_ok() { "success" } else { "error" };
        info!(
            metrics = true,
            histogram.layer_cache_persister_evict_memory_only_duration_seconds = duration,
            cache_name = &cache_name,
            status = status
        );

        result
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn evict_from_pg(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        let pg_layer = PgLayer::new(self.pg_pool.clone(), event.payload.db_name.as_ref());
        pg_layer.delete(&event.payload.key).await?;
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn write_layers(self, event: LayeredEvent, status_tx: PersisterStatusWriter) {
        match self.try_write_layers(event, false).await {
            Ok(_) => status_tx.send(PersistStatus::Finished),
            Err(err) => {
                error!(error = ?err, "persister write task failed");
                status_tx.send(PersistStatus::Error(err));
            }
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn try_write_layers(&self, event: LayeredEvent, is_retry: bool) -> LayerDbResult<()> {
        let start = std::time::Instant::now();
        let cache_name = event.payload.db_name.to_string();
        let event = Arc::new(event);

        // Write to nats
        let nats_join = if is_retry {
            None
        } else {
            Some(self.layered_event_client.publish(event.clone()).await?)
        };

        // Write to pg
        let pg_self = self.clone();
        let pg_event = event.clone();
        let pg_join = tokio::task::spawn(async move { pg_self.write_to_pg(pg_event).await });

        // Wait for PostgreSQL write
        let pg_result = pg_join.await;
        let pg_error = match &pg_result {
            Ok(Ok(_)) => None,
            Ok(Err(e)) => Some(e.to_string()),
            Err(e) => Some(e.to_string()),
        };

        // Wait for NATS publish if this was an initial attempt
        let nats_error = if let Some(nats_handle) = nats_join {
            match nats_handle.await {
                Ok(Ok(_)) => None,
                Ok(Err(e)) => Some(e.to_string()),
                Err(e) => Some(e.to_string()),
            }
        } else {
            None
        };

        let result = if pg_error.is_some() || nats_error.is_some() {
            let kind = PersisterTaskErrorKind::Write;

            // Track which component failed
            if pg_error.is_some() && nats_error.is_some() {
                info!(
                    metrics = true,
                    counter.layer_cache_persister_both_error = 1,
                    cache_name = &cache_name,
                    operation = "write"
                );
            } else if pg_error.is_some() {
                info!(
                    metrics = true,
                    counter.layer_cache_persister_pg_error = 1,
                    cache_name = &cache_name,
                    operation = "write"
                );
            } else if nats_error.is_some() {
                info!(
                    metrics = true,
                    counter.layer_cache_persister_nats_error = 1,
                    cache_name = &cache_name,
                    operation = "write"
                );
            }

            Err(LayerDbError::PersisterTaskFailed(PersisterTaskError {
                kind,
                pg_error,
                nats_error,
            }))
        } else {
            Ok(())
        };

        let duration = start.elapsed().as_secs_f64();
        let status = if result.is_ok() { "success" } else { "error" };
        info!(
            metrics = true,
            histogram.layer_cache_persister_write_duration_seconds = duration,
            cache_name = &cache_name,
            status = status
        );

        result
    }

    // Write an event to the pg layer
    #[instrument(level = "debug", skip_all)]
    pub async fn write_to_pg(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        let cache_name = event.payload.db_name.to_string();
        let event_kind = format!("{:?}", event.event_kind);

        info!(
            metrics = true,
            counter.layer_cache_persister_event_by_kind = 1,
            cache_name = &cache_name,
            event_kind = &event_kind
        );

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
