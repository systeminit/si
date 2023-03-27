use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;
use ulid::Ulid;

use super::consumer::{JobConsumerCustomPayload, JobConsumerMetadata, JobInfo};

#[derive(Error, Debug)]
pub enum JobProducerError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("arg {0:?} not found at index {1}")]
    ArgNotFound(JobInfo, usize),
}

pub type JobProducerResult<T> = Result<T, JobProducerError>;

pub trait JobProducer: std::fmt::Debug + Send + JobConsumerMetadata {
    fn args(&self) -> JobProducerResult<serde_json::Value>;
    fn meta(&self) -> JobProducerResult<JobMeta>;
    fn identity(&self) -> String;
}

pub type BlockingJobResult = Result<(), BlockingJobError>;

#[derive(Error, Clone, Debug, Serialize, Deserialize)]
pub enum BlockingJobError {
    #[error("A nats error occurred: {0}")]
    Nats(String),
    #[error("Error during job execution: {0}")]
    JobExecution(String),
    #[error("JobProducer error: {0}")]
    JobProducer(String),
    #[error("serde error: {0}")]
    Serde(String),
    #[error("A transactions error occurred: {0}")]
    Transactions(String),
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct JobMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<isize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default = "HashMap::default")]
    pub custom: HashMap<String, serde_json::Value>,
}

impl TryFrom<Box<dyn JobProducer + Send + Sync>> for JobInfo {
    type Error = JobProducerError;

    fn try_from(job_producer: Box<dyn JobProducer + Send + Sync>) -> Result<Self, Self::Error> {
        let job_producer_meta = job_producer.meta()?;

        Ok(JobInfo {
            id: Ulid::new().to_string(),
            kind: job_producer.type_name(),
            queue: None,
            created_at: Some(Utc::now()),
            enqueued_at: None,
            at: job_producer_meta.at,
            args: vec![
                job_producer.args()?,
                serde_json::to_value(job_producer.access_builder())?,
                serde_json::to_value(job_producer.visibility())?,
            ],
            retry: job_producer_meta.retry,
            custom: JobConsumerCustomPayload {
                extra: job_producer_meta.custom,
            },
        })
    }
}

impl JobProducer for JobInfo {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        self.args
            .get(0)
            .cloned()
            .ok_or(JobProducerError::ArgNotFound(self.clone(), 0))
    }
    fn meta(&self) -> JobProducerResult<JobMeta> {
        Ok(JobMeta {
            at: self.at,
            retry: self.retry,
            custom: self.custom.extra.clone(),
        })
    }

    fn identity(&self) -> String {
        serde_json::to_string(&serde_json::json!({
            "args": self.args,
            "kind": self.kind,
        }))
        .expect("Cannot serialize JobInfo")
    }
}
