use async_trait::async_trait;
use dyn_clone::DynClone;
use thiserror::Error;

use crate::{
    job::producer::{BlockingJobError, BlockingJobResult, JobProducer, JobProducerError},
    job::queue::JobQueue,
};

mod nats_processor;
pub use nats_processor::NatsProcessor;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum JobQueueProcessorError {
    #[error("Error processing blocking job: {0}")]
    BlockingJob(#[from] BlockingJobError),
    #[error(transparent)]
    JobProducer(#[from] JobProducerError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Transport(Box<dyn std::error::Error + Sync + Send + 'static>),
}

pub type JobQueueProcessorResult<T> = Result<T, JobQueueProcessorError>;

#[async_trait]
pub trait JobQueueProcessor: std::fmt::Debug + DynClone {
    async fn block_on_job(&self, job: Box<dyn JobProducer + Send + Sync>) -> BlockingJobResult;
    async fn block_on_jobs(
        &self,
        jobs: Vec<Box<dyn JobProducer + Send + Sync>>,
    ) -> BlockingJobResult;
    async fn process_queue(&self, queue: JobQueue) -> JobQueueProcessorResult<()>;
    async fn blocking_process_queue(&self, queue: JobQueue) -> JobQueueProcessorResult<()>;
}

dyn_clone::clone_trait_object!(JobQueueProcessor);
