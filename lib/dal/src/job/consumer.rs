use std::time::Duration;

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

use crate::prop::PropError;
use crate::validation::ValidationError;
use crate::{
    action::prototype::ActionPrototypeError, action::ActionError,
    attribute::value::AttributeValueError,
    job::definition::dependent_values_update::DependentValueUpdateError,
    job::producer::BlockingJobError, job::producer::JobProducerError, AccessBuilder,
    ActionPrototypeId, ComponentError, ComponentId, DalContext, DalContextBuilder,
    StandardModelError, TransactionsError, Visibility, WorkspaceSnapshotError, WsEventError,
};

#[remain::sorted]
#[derive(Error, Debug)]
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
    #[error("Error blocking on job: {0}")]
    BlockingJob(#[from] BlockingJobError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component {0} is destroyed")]
    ComponentIsDestroyed(ComponentId),
    #[error("dependent value update error: {0}")]
    DependentValueUpdate(#[from] DependentValueUpdateError),
    #[error("Invalid job arguments. Expected: {0} Actual: {1:?}")]
    InvalidArguments(String, Vec<Value>),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    JobProducer(#[from] JobProducerError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error("nats is unavailable")]
    NatsUnavailable,
    // #[error("no schema found for component {0}")]
    // NoSchemaFound(ComponentId),
    // #[error("no schema variant found for component {0}")]
    // NoSchemaVariantFound(ComponentId),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    Prop(#[from] PropError),
    // #[error(transparent)]
    // PropertyEditorValuesSummary(#[from] PropertyEditorValuesSummaryError),
    #[error("execution of job {0} failed after {1} retry attempts")]
    RetriesFailed(String, u32),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

impl From<JobConsumerError> for std::io::Error {
    fn from(jce: JobConsumerError) -> Self {
        Self::new(std::io::ErrorKind::InvalidData, jce)
    }
}

pub type JobConsumerResult<T> = Result<T, JobConsumerError>;

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
                        return Err(JobConsumerError::RetriesFailed(self.type_name(), retries));
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
