use async_trait::async_trait;
use dyn_clone::DynClone;
use si_data_nats::async_nats;
use thiserror::Error;

use crate::job::{
    consumer::DalJob,
    producer::{
        BlockingJobError,
        BlockingJobResult,
    },
    queue::JobQueue,
};

mod nats_processor;
pub use nats_processor::NatsProcessor;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum JobQueueProcessorError {
    #[error("Error processing blocking job: {0}")]
    BlockingJob(#[from] BlockingJobError),
    #[error("stream create error: {0}")]
    JsCreateStreamError(#[from] async_nats::jetstream::context::CreateStreamError),
    #[error("missing required workspace_pk")]
    MissingWorkspacePk,
    #[error("pinga client error: {0}")]
    PingaClient(#[from] Box<pinga_client::ClientError>),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Transport(Box<dyn std::error::Error + Sync + Send + 'static>),
}

impl From<pinga_client::ClientError> for JobQueueProcessorError {
    fn from(value: pinga_client::ClientError) -> Self {
        Box::new(value).into()
    }
}

pub type JobQueueProcessorResult<T> = Result<T, JobQueueProcessorError>;

#[async_trait]
pub trait JobQueueProcessor: std::fmt::Debug + DynClone {
    async fn block_on_job(&self, job: Box<dyn DalJob>) -> BlockingJobResult;
    async fn block_on_jobs(&self, jobs: Vec<Box<dyn DalJob>>) -> BlockingJobResult;
    async fn process_queue(&self, queue: JobQueue) -> JobQueueProcessorResult<()>;
    async fn blocking_process_queue(&self, queue: JobQueue) -> JobQueueProcessorResult<()>;
}

dyn_clone::clone_trait_object!(JobQueueProcessor);
