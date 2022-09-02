use async_trait::async_trait;
use dyn_clone::DynClone;
use thiserror::Error;

use super::producer::{JobProducer, JobProducerError};
use crate::DalContext;

pub mod faktory_processor;
pub mod sync_processor;

#[derive(Error, Debug)]
pub enum JobQueueProcessorError {
    #[error(transparent)]
    Faktory(#[from] faktory_async::Error),
    #[error(transparent)]
    JobProducer(#[from] JobProducerError),
}

pub type JobQueueProcessorResult<T> = Result<T, JobQueueProcessorError>;

#[async_trait]
pub trait JobQueueProcessor: std::fmt::Debug + DynClone {
    async fn enqueue_job(
        &self,
        job: Box<dyn JobProducer + Send + Sync>,
        ctx: &DalContext<'_, '_, '_>,
    );
    async fn process_queue(&self) -> JobQueueProcessorResult<()>;
}

dyn_clone::clone_trait_object!(JobQueueProcessor);
