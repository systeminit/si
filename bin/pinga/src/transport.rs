use async_trait::async_trait;
use faktory_async::{BeatState, FailConfig};
use futures::StreamExt;
use si_data_faktory::FaktoryConfig;
use si_data_nats::{NatsClient, NatsConfig};
use telemetry_application::prelude::*;
use tokio::sync::{mpsc, watch};

use crate::JobError;
use dal::{job::consumer::JobInfo, FaktoryProcessor, JobQueueProcessor, NatsProcessor};

pub enum ExecutionState {
    Process(JobInfo),
    Idle,
    Stop,
}

#[async_trait]
pub trait Consumer: Sized {
    type Client: Sized;
    type ClientConfig;

    async fn connect(config: &Self::ClientConfig) -> Result<Self::Client, JobError>;
    fn new_processor(
        client: Self::Client,
        alive_marker: mpsc::Sender<()>,
    ) -> Box<dyn JobQueueProcessor + Send + Sync>;
    async fn end(client: Self::Client) -> Result<(), JobError>;
    async fn subscribe(client: &Self::Client) -> Result<Self, JobError>;
    async fn fetch_next(&mut self, shutdown_request_rx: &mut watch::Receiver<()>)
        -> ExecutionState;
    async fn post_process(&self, jid: String, result: Result<(), JobError>);
}

#[async_trait]
impl Consumer for faktory_async::Client {
    type Client = Self;
    type ClientConfig = FaktoryConfig;

    async fn connect(config: &Self::ClientConfig) -> Result<Self::Client, JobError> {
        let config = faktory_async::Config::from_uri(
            &config.url,
            Some("pinga".to_string()),
            Some(uuid::Uuid::new_v4().to_string()),
        );
        Ok(faktory_async::Client::new(config, 256))
    }
    fn new_processor(
        client: Self::Client,
        alive_marker: mpsc::Sender<()>,
    ) -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(FaktoryProcessor::new(client, alive_marker))
            as Box<dyn JobQueueProcessor + Send + Sync>
    }
    async fn end(client: Self::Client) -> Result<(), JobError> {
        Ok(client.close().await?)
    }
    async fn subscribe(client: &Self::Client) -> Result<Self, JobError> {
        Ok(client.clone())
    }
    async fn fetch_next(
        &mut self,
        shutdown_request_rx: &mut watch::Receiver<()>,
    ) -> ExecutionState {
        match self.last_beat().await {
            Ok(BeatState::Ok) => {
                tokio::select! {
                    job = self.fetch(vec!["default".into()]) => match job {
                        Ok(Some(job)) => match JobInfo::try_from(job) {
                            Ok(job) => ExecutionState::Process(job),
                            Err(err) => {
                                error!("Unable to deserialize faktory's job: {err}");
                                ExecutionState::Idle
                            }
                        },
                        Ok(None) => ExecutionState::Idle,
                        Err(err) => {
                            error!("Unable to fetch from faktory: {err}");
                            ExecutionState::Idle
                        }
                    },
                    _ = shutdown_request_rx.changed() => {
                        info!("Worker task received shutdown notification: stopping");
                        ExecutionState::Stop
                    }
                }
            }
            Ok(BeatState::Quiet) => {
                // Getting a "quiet" state from the faktory server means that
                // someone has gone to the faktory UI and requested that this
                // particular worker finish what it's doing, and gracefully
                // shut down.
                info!("Gracefully shutting down from Faktory request.");
                ExecutionState::Stop
            }
            Ok(BeatState::Terminate) => {
                warn!("Faktory asked us to terminate");
                ExecutionState::Stop
            }
            Err(err) => {
                error!("Internal error in faktory-async, bailing out: {err}");
                ExecutionState::Stop
            }
        }
    }
    async fn post_process(&self, jid: String, result: Result<(), JobError>) {
        match result {
            Ok(()) => match self.ack(jid).await {
                Ok(()) => {}
                Err(err) => {
                    error!("Ack failed: {err}");
                }
            },
            Err(err) => {
                error!("Job execution failed: {err}");
                // TODO: pass backtrace here
                match self
                    .fail(FailConfig::new(
                        jid,
                        format!("{err:?}"),
                        err.to_string(),
                        None,
                    ))
                    .await
                {
                    Ok(()) => {}
                    Err(err) => error!("Fail failed: {err}"),
                }
            }
        }
    }
}

#[async_trait]
impl Consumer for si_data_nats::Subscription {
    type Client = NatsClient;
    type ClientConfig = NatsConfig;

    async fn connect(config: &Self::ClientConfig) -> Result<Self::Client, JobError> {
        Ok(NatsClient::new(config).await?)
    }
    fn new_processor(
        client: Self::Client,
        alive_marker: mpsc::Sender<()>,
    ) -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(NatsProcessor::new(client, alive_marker))
            as Box<dyn JobQueueProcessor + Send + Sync>
    }
    async fn end(client: Self::Client) -> Result<(), JobError> {
        Ok(client.close().await?)
    }
    async fn subscribe(client: &Self::Client) -> Result<Self, JobError> {
        Ok(client.queue_subscribe("pinga-jobs", "pinga").await?)
    }
    async fn fetch_next(
        &mut self,
        shutdown_request_rx: &mut watch::Receiver<()>,
    ) -> ExecutionState {
        tokio::select! {
            job = self.next() => match job {
                None => ExecutionState::Idle,
                Some(result) => match result {
                    Ok(msg) => match serde_json::from_slice::<JobInfo>(msg.data()) {
                        Ok(job) => ExecutionState::Process(job),
                        Err(err) => {
                            error!("Unable to deserialize nats' job: {err}");
                            ExecutionState::Idle
                        }
                    }
                    Err(err) => {
                        error!("Internal error in nats, bailing out: {err}");
                        ExecutionState::Stop
                    }
                }
            },
            _ = shutdown_request_rx.changed() => {
                info!("Worker task received shutdown notification: stopping");
                ExecutionState::Stop
            }
        }
    }
    async fn post_process(&self, jid: String, result: Result<(), JobError>) {
        match result {
            Ok(()) => {}
            Err(err) => error!("Job {jid} execution failed: {err}"),
        }
    }
}
