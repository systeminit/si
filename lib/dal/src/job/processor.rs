use async_trait::async_trait;
use dyn_clone::DynClone;
use thiserror::Error;

use super::producer::{JobProducer, JobProducerError};
use crate::DalContext;

pub mod nats_processor;

#[derive(Error, Debug)]
pub enum JobQueueProcessorError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Transport(Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error(transparent)]
    JobProducer(#[from] JobProducerError),
}

pub type JobQueueProcessorResult<T> = Result<T, JobQueueProcessorError>;

#[async_trait]
pub trait JobQueueProcessor: std::fmt::Debug + DynClone {
    async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>, ctx: &DalContext);
    async fn enqueue_blocking_job(&self, job: Box<dyn JobProducer + Send + Sync>, ctx: &DalContext);
    async fn process_queue(&self) -> JobQueueProcessorResult<()>;
}

dyn_clone::clone_trait_object!(JobQueueProcessor);
