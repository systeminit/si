use async_trait::async_trait;
use dal::{job::consumer::JobInfo, JobQueueProcessor, NatsProcessor};
use futures::StreamExt;
use si_data_nats::NatsClient;
use telemetry_application::prelude::*;
use tokio::sync::{mpsc, watch};

use crate::{config, JobError};

pub enum ExecutionState {
    Process(JobInfo),
    Idle,
    Stop,
}

#[async_trait]
pub trait Consumer: Sized {
    type Client: Sized;

    async fn connect(config: &config::Config) -> Result<Self::Client, JobError>;
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
impl Consumer for si_data_nats::Subscription {
    type Client = NatsClient;

    async fn connect(config: &config::Config) -> Result<Self::Client, JobError> {
        Ok(NatsClient::new(config.nats()).await?)
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
