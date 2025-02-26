use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_nats::NatsError;
use si_data_pg::PgPoolError;
use si_layer_cache::LayerDbError;
use thiserror::Error;
use tokio::task::JoinError;

use crate::{
    action::{prototype::ActionPrototypeError, ActionError},
    attribute::value::AttributeValueError,
    billing_publish::BillingPublishError,
    diagram::DiagramError,
    job::{
        definition::dependent_values_update::DependentValueUpdateError,
        producer::{BlockingJobError, JobProducerError},
    },
    prop::PropError,
    validation::ValidationError,
    AccessBuilder, ActionPrototypeId, ChangeSetError, ComponentError, ComponentId, DalContext,
    DalContextBuilder, FuncError, StandardModelError, TransactionsError, Visibility,
    WorkspaceSnapshotError, WsEventError,
};

#[remain::sorted]
#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum JobConsumerError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("ActionProtoype {0} not found")]
    ActionPrototypeNotFound(ActionPrototypeId),
    #[error("arg {0:?} not found at index {1}")]
    ArgNotFound(JobInfo, usize),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("billing publish error: {0}")]
    BillingPublish(#[from] BillingPublishError),
    #[error("Error blocking on job: {0}")]
    BlockingJob(#[from] BlockingJobError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component {0} is destroyed")]
    ComponentIsDestroyed(ComponentId),
    #[error("dependent value update error: {0}")]
    DependentValueUpdate(#[from] DependentValueUpdateError),
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("Invalid job arguments. Expected: {0} Actual: {1:?}")]
    InvalidArguments(String, Vec<Value>),
    #[error("std io error: {0}")]
    Io(#[from] ::std::io::Error),
    #[error("job producer error: {0}")]
    JobProducer(#[from] JobProducerError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("nats error: {0}")]
    Nats(#[from] NatsError),
    #[error("nats is unavailable")]
    NatsUnavailable,
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("execution of job {0} failed after {1} retry attempts")]
    RetriesFailed(String, u32),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("tokio task error: {0}")]
    TokioTask(#[from] JoinError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

impl From<JobConsumerError> for std::io::Error {
    fn from(jce: JobConsumerError) -> Self {
        Self::new(std::io::ErrorKind::InvalidData, jce)
    }
}

pub type JobConsumerResult<T> = Result<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub kind: String,
    pub created_at: DateTime<Utc>,
    pub arg: Value,
    pub access_builder: AccessBuilder,
    pub visibility: Visibility,
    pub blocking: bool,
}

pub enum RetryBackoff {
    Exponential,
    None,
}

/// Jobs that return a state of `JobCompletionState::Retry` will be retried
/// with the requested backoff and limit
pub enum JobCompletionState {
    Retry { limit: u32, backoff: RetryBackoff },
    Done,
}

#[async_trait]
pub trait JobConsumerMetadata: std::fmt::Debug + Sync {
    fn type_name(&self) -> String;
    fn access_builder(&self) -> AccessBuilder;
    fn visibility(&self) -> Visibility;
}

#[async_trait]
// Having Sync as a supertrait gets around triggering https://github.com/rust-lang/rust/issues/51443
pub trait JobConsumer: std::fmt::Debug + Sync + JobConsumerMetadata {
    /// Intended to be defined by implementations of this trait.
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState>;

    /// Called on the trait object to set up the data necessary to run the job,
    /// and in-turn calls the `run` method. Can be overridden by an implementation
    /// of the trait if you need more control over how the `DalContext` is managed
    /// during the lifetime of the job.
    async fn run_job(&self, ctx_builder: DalContextBuilder) -> JobConsumerResult<()> {
        let mut retries = 0;
        loop {
            let mut ctx = ctx_builder
                .build(self.access_builder().build(self.visibility()))
                .await?;

            match self.run(&mut ctx).await? {
                JobCompletionState::Retry { limit, backoff } => {
                    if retries >= limit {
                        return Err(
                            JobConsumerError::RetriesFailed(self.type_name(), retries).into()
                        );
                    }

                    if let RetryBackoff::Exponential = backoff {
                        tokio::time::sleep(calculate_exponential_sleep_ms(retries, 2)).await;
                    };
                }
                JobCompletionState::Done => {
                    break;
                }
            }

            retries = retries.saturating_add(1);
        }

        Ok(())
    }
}

fn calculate_exponential_sleep_ms(retry_no: u32, base: u32) -> Duration {
    let sleep_micros = base.pow(retry_no).saturating_mul(1000);
    let mut rng = rand::thread_rng();
    // "full" jitter, to prevent "thundering herd". On average this still gives
    // us an exponential distribution
    let jittered_micros = rng.gen_range(1000..=sleep_micros);

    Duration::from_micros(jittered_micros.into())
}
