use crate::{server::server::ServerError, Config, Server};
use async_trait::async_trait;
use dal::{FaktoryProcessor, JobQueueProcessor, NatsProcessor};
use si_data_nats::NatsClient;
use tokio::sync::mpsc;

#[async_trait]
pub trait JobProcessorClientCloser {
    async fn close(&self) -> Result<(), ServerError>;
}

#[async_trait]
impl JobProcessorClientCloser for NatsClient {
    async fn close(&self) -> Result<(), ServerError> {
        Ok(self.close().await?)
    }
}

#[async_trait]
impl JobProcessorClientCloser for faktory_async::Client {
    async fn close(&self) -> Result<(), ServerError> {
        Ok(self.close().await?)
    }
}

#[async_trait]
pub trait JobProcessorConnector: JobQueueProcessor {
    type Client: JobProcessorClientCloser;

    async fn connect(
        config: &Config,
        alive_marker: mpsc::Sender<()>,
    ) -> Result<(Self::Client, Box<dyn JobQueueProcessor + Send + Sync>), ServerError>;
}

#[async_trait]
impl JobProcessorConnector for FaktoryProcessor {
    type Client = faktory_async::Client;

    async fn connect(
        config: &Config,
        alive_marker: mpsc::Sender<()>,
    ) -> Result<(Self::Client, Box<dyn JobQueueProcessor + Send + Sync>), ServerError> {
        let job_client = faktory_async::Client::new(
            faktory_async::Config::from_uri(&config.faktory().url, Some("sdf".to_string()), None),
            256,
        );
        let job_processor = Box::new(FaktoryProcessor::new(job_client.clone(), alive_marker))
            as Box<dyn JobQueueProcessor + Send + Sync>;
        Ok((job_client, job_processor))
    }
}

#[async_trait]
impl JobProcessorConnector for NatsProcessor {
    type Client = NatsClient;

    async fn connect(
        config: &Config,
        alive_marker: mpsc::Sender<()>,
    ) -> Result<(Self::Client, Box<dyn JobQueueProcessor + Send + Sync>), ServerError> {
        let job_client = Server::connect_to_nats(config.nats()).await?;
        let job_processor = Box::new(NatsProcessor::new(job_client.clone(), alive_marker))
            as Box<dyn JobQueueProcessor + Send + Sync>;
        Ok((job_client, job_processor))
    }
}
