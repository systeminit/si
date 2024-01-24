use crate::{server::server::ServerError, Config, Server};
use async_trait::async_trait;
use dal::{JobQueueProcessor, NatsProcessor};
use si_data_nats::NatsClient;

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
pub trait JobProcessorConnector: JobQueueProcessor {
    type Client: JobProcessorClientCloser;

    async fn connect(
        config: &Config,
    ) -> Result<(Self::Client, Box<dyn JobQueueProcessor + Send + Sync>), ServerError>;
}

#[async_trait]
impl JobProcessorConnector for NatsProcessor {
    type Client = NatsClient;

    async fn connect(
        config: &Config,
    ) -> Result<(Self::Client, Box<dyn JobQueueProcessor + Send + Sync>), ServerError> {
        let job_client = Server::connect_to_nats(config.nats()).await?;
        dbg!(&job_client);
        let job_processor = Box::new(NatsProcessor::new(job_client.clone()))
            as Box<dyn JobQueueProcessor + Send + Sync>;
        Ok((job_client, job_processor))
    }
}
