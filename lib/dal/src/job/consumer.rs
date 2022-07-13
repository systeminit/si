use std::{collections::HashMap, convert::TryFrom, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data::PgPoolError;
use thiserror::Error;

use crate::{
    AccessBuilder, AttributeValueError, ComponentError, DalContext, DalContextBuilder,
    StandardModelError, TransactionsError, Visibility,
};

#[derive(Error, Debug)]
pub enum JobConsumerError {
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("Invalid job arguments. Expected: {0} Actual: {1:?}")]
    InvalidArguments(String, Vec<Value>),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

impl From<JobConsumerError> for std::io::Error {
    fn from(jce: JobConsumerError) -> Self {
        Self::new(std::io::ErrorKind::InvalidData, jce)
    }
}

pub type JobConsumerResult<T> = Result<T, JobConsumerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaktoryJobInfo {
    pub id: String,
    pub kind: String,
    pub queue: String,
    pub created_at: Option<DateTime<Utc>>,
    pub enqueued_at: Option<DateTime<Utc>>,
    pub at: Option<DateTime<Utc>>,
    pub args: Vec<Value>,
    pub custom: JobConsumerCustomPayload,
}

impl TryFrom<faktory::Job> for FaktoryJobInfo {
    type Error = JobConsumerError;

    fn try_from(job: faktory::Job) -> Result<Self, Self::Error> {
        let custom: JobConsumerCustomPayload =
            serde_json::from_value(serde_json::to_value(job.custom.clone())?)?;

        Ok(FaktoryJobInfo {
            id: job.id().to_string(),
            kind: job.kind().to_string(),
            queue: job.queue.clone(),
            created_at: job.created_at,
            enqueued_at: job.enqueued_at,
            at: job.at,
            args: job.args().to_vec(),
            custom,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobConsumerCustomPayload {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[async_trait]
// Having Sync as a supertrait gets around triggering https://github.com/rust-lang/rust/issues/51443
pub trait JobConsumer: std::fmt::Debug + Sync {
    fn type_name(&self) -> String;
    fn access_builder(&self) -> AccessBuilder;
    fn visibility(&self) -> Visibility;

    async fn run(&self, ctx: &DalContext<'_, '_>) -> JobConsumerResult<()>;

    async fn run_job(&self, ctx_builder: Arc<DalContextBuilder>) -> JobConsumerResult<()> {
        let mut txns = ctx_builder.transactions_starter().await?;
        let txns = txns.start().await?;
        let ctx = ctx_builder.build(self.access_builder().build(self.visibility()), &txns);

        self.run(&ctx).await?;

        txns.commit().await?;

        Ok(())
    }
}
