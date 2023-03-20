use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{collections::HashMap, collections::VecDeque, convert::TryFrom};
use thiserror::Error;
use ulid::Ulid;

use super::consumer::{JobConsumerCustomPayload, JobConsumerMetadata, JobInfo};
use crate::{AccessBuilder, Visibility};

#[derive(Error, Debug)]
pub enum JobProducerError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("arg not found at index {1} for job {0:?}")]
    ArgNotFound(JobInfo, usize),
}

pub type JobProducerResult<T> = Result<T, JobProducerError>;

pub trait JobProducer: std::fmt::Debug + JobConsumerMetadata {
    fn args(&self) -> JobProducerResult<serde_json::Value>;
    fn meta(&self) -> JobProducerResult<JobMeta>;
    fn identity(&self) -> String;
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

impl JobConsumerMetadata for JobInfo {
    fn type_name(&self) -> String {
        self.kind.clone()
    }

    fn access_builder(&self) -> AccessBuilder {
        serde_json::from_value(
            self.args
                .get(1)
                .cloned()
                .ok_or(JobProducerError::ArgNotFound(self.clone(), 1))
                .expect("access builder not found"),
        )
        .expect("unable to serialize access builder")
    }

    fn visibility(&self) -> Visibility {
        serde_json::from_value(
            self.args
                .get(2)
                .cloned()
                .ok_or(JobProducerError::ArgNotFound(self.clone(), 2))
                .expect("visibility not found"),
        )
        .expect("unable to serialize visibility")
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
            subsequent_jobs: VecDeque::new(),
        })
    }
}
