use async_trait::async_trait;
use dyn_clone::DynClone;
use thiserror::Error;

use crate::{
    job::producer::{BlockingJobError, BlockingJobResult, JobProducer, JobProducerError},
    DalContext,
};

mod nats_processor;
pub use nats_processor::NatsProcessor;

#[derive(Error, Debug)]
pub enum JobQueueProcessorError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Transport(Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error(transparent)]
    JobProducer(#[from] JobProducerError),
    #[error("Error processing blocking job: {0}")]
    BlockingJob(#[from] BlockingJobError),
}

pub type JobQueueProcessorResult<T> = Result<T, JobQueueProcessorError>;

#[async_trait]
pub trait JobQueueProcessor: std::fmt::Debug + DynClone {
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>, ctx: &DalContext);
    async fn block_on_job(&self, job: Box<dyn JobProducer + Send + Sync>) -> BlockingJobResult;
    async fn block_on_jobs(
        &self,
        jobs: Vec<Box<dyn JobProducer + Send + Sync>>,
    ) -> BlockingJobResult;
    async fn process_queue(&self) -> JobQueueProcessorResult<()>;
    async fn blocking_process_queue(&self) -> JobQueueProcessorResult<()>;
}

dyn_clone::clone_trait_object!(JobQueueProcessor);
