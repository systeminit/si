use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

use super::consumer::{JobConsumerMetadata, JobInfo};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum JobProducerError {
    #[error("arg {0:?} not found at index {1}")]
    ArgNotFound(JobInfo, usize),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type JobProducerResult<T> = Result<T, JobProducerError>;

pub trait JobProducer: std::fmt::Debug + Send + JobConsumerMetadata {
    fn arg(&self) -> JobProducerResult<serde_json::Value>;
}

pub type BlockingJobResult = Result<(), BlockingJobError>;

#[remain::sorted]
#[derive(Error, Clone, Debug, Serialize, Deserialize)]
pub enum BlockingJobError {
    #[error("Error during job execution: {0}")]
    JobExecution(String),
    #[error("JobProducer error: {0}")]
    JobProducer(String),
    #[error("A nats error occurred: {0}")]
    Nats(String),
    #[error("no access builder found in job info")]
    NoAccessBuilder,
    #[error("serde error: {0}")]
    Serde(String),
    #[error("A transactions error occurred: {0}")]
    Transactions(String),
}

impl JobInfo {
    pub fn new(job_producer: Box<dyn JobProducer + Send + Sync>) -> JobProducerResult<Self> {
        Ok(Self {
            id: Ulid::new().to_string(),
            kind: job_producer.type_name(),
            created_at: Utc::now(),
            arg: job_producer.arg()?,
            access_builder: job_producer.access_builder(),
            visibility: job_producer.visibility(),
            blocking: false,
        })
    }

    pub fn new_blocking(
        job_producer: Box<dyn JobProducer + Send + Sync>,
    ) -> JobProducerResult<Self> {
        Ok(Self {
            id: Ulid::new().to_string(),
            kind: job_producer.type_name(),
            created_at: Utc::now(),
            arg: job_producer.arg()?,
            access_builder: job_producer.access_builder(),
            visibility: job_producer.visibility(),
            blocking: true,
        })
    }
}
