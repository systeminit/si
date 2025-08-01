use std::time::Duration;

use async_trait::async_trait;
use pinga_core::api_types::job_execution_request::JobArgsVCurrent;
use rand::Rng;
use serde_json::Value;
use si_data_nats::NatsError;
use si_data_pg::PgPoolError;
use si_db::HistoryActor;
use si_events::authentication_method::AuthenticationMethodV1;
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use thiserror::Error;
use tokio::task::JoinError;

use crate::{
    AccessBuilder,
    ActionPrototypeId,
    ChangeSetError,
    ComponentError,
    ComponentId,
    DalContext,
    DalContextBuilder,
    FuncError,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEventError,
    action::{
        ActionError,
        prototype::ActionPrototypeError,
    },
    attribute::value::AttributeValueError,
    billing_publish::BillingPublishError,
    diagram::DiagramError,
    func::runner::FuncRunnerError,
    job::{
        definition::{
            ManagementFuncJobError,
            dependent_values_update::DependentValueUpdateError,
        },
        producer::BlockingJobError,
    },
    prop::PropError,
    validation::ValidationError,
};

#[remain::sorted]
#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum JobConsumerError {
    #[error("action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("ActionProtoype {0} not found")]
    ActionPrototypeNotFound(ActionPrototypeId),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("billing publish error: {0}")]
    BillingPublish(#[from] BillingPublishError),
    #[error("Error blocking on job: {0}")]
    BlockingJob(#[from] Box<BlockingJobError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("component {0} is destroyed")]
    ComponentIsDestroyed(ComponentId),
    #[error("dependent value update error: {0}")]
    DependentValueUpdate(#[from] Box<DependentValueUpdateError>),
    #[error("diagram error: {0}")]
    Diagram(#[from] Box<DiagramError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] Box<FuncRunnerError>),
    #[error("Invalid job arguments. Expected: {0} Actual: {1:?}")]
    InvalidArguments(String, Vec<Value>),
    #[error("std io error: {0}")]
    Io(#[from] ::std::io::Error),
    #[error("management function error: {0}")]
    ManagementFunc(#[from] Box<ManagementFuncJobError>),
    #[error("nats error: {0}")]
    Nats(#[from] NatsError),
    #[error("nats is unavailable")]
    NatsUnavailable,
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("execution of job {0} failed after {1} retry attempts")]
    RetriesFailed(JobArgsVCurrent, u32),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("tokio task error: {0}")]
    TokioTask(#[from] JoinError),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("validation error: {0}")]
    Validation(#[from] Box<ValidationError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<JobConsumerError> for std::io::Error {
    fn from(jce: JobConsumerError) -> Self {
        Self::new(std::io::ErrorKind::InvalidData, jce)
    }
}

impl From<ActionError> for JobConsumerError {
    fn from(value: ActionError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for JobConsumerError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for JobConsumerError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<DiagramError> for JobConsumerError {
    fn from(value: DiagramError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for JobConsumerError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncRunnerError> for JobConsumerError {
    fn from(value: FuncRunnerError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for JobConsumerError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}

impl From<ActionPrototypeError> for JobConsumerError {
    fn from(value: ActionPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<BlockingJobError> for JobConsumerError {
    fn from(value: BlockingJobError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for JobConsumerError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<DependentValueUpdateError> for JobConsumerError {
    fn from(value: DependentValueUpdateError) -> Self {
        Box::new(value).into()
    }
}

impl From<ManagementFuncJobError> for JobConsumerError {
    fn from(value: ManagementFuncJobError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for JobConsumerError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for JobConsumerError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<ValidationError> for JobConsumerError {
    fn from(value: ValidationError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for JobConsumerError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

pub type JobConsumerResult<T> = Result<T, JobConsumerError>;

pub enum RetryBackoff {
    Exponential,
    None,
}

/// Jobs that return a state of `JobCompletionState::Retry` will be retried
/// with the requested backoff and limit
pub enum JobCompletionState {
    Retry {
        limit: u32,
        backoff: RetryBackoff,
        minimum: u32,
    },
    Done,
}

#[async_trait]
pub trait DalJob: std::fmt::Debug + Sync + Send {
    fn args(&self) -> JobArgsVCurrent;
    fn workspace_id(&self) -> WorkspacePk;
    fn change_set_id(&self) -> ChangeSetId;
}

#[async_trait]
// Having Sync as a supertrait gets around triggering https://github.com/rust-lang/rust/issues/51443
pub trait JobConsumer: std::fmt::Debug + Sync + DalJob {
    /// Intended to be defined by implementations of this trait.
    async fn run(&self, ctx: &mut DalContext) -> JobConsumerResult<JobCompletionState>;

    /// Called on the trait object to set up the data necessary to run the job,
    /// and in-turn calls the `run` method. Can be overridden by an implementation
    /// of the trait if you need more control over how the `DalContext` is managed
    /// during the lifetime of the job.
    async fn run_job(&self, ctx_builder: DalContextBuilder) -> JobConsumerResult<()> {
        let request_context = AccessBuilder::new(
            self.workspace_id().into(),
            HistoryActor::SystemInit,
            None,
            AuthenticationMethodV1::System,
        )
        .build(self.change_set_id().into());

        let mut retries = 0;
        loop {
            let mut ctx = ctx_builder.build(request_context.clone()).await?;

            match self.run(&mut ctx).await? {
                JobCompletionState::Retry {
                    limit,
                    minimum,
                    backoff,
                } => {
                    if retries >= limit {
                        return Err(JobConsumerError::RetriesFailed(self.args(), retries));
                    }

                    if let RetryBackoff::Exponential = backoff {
                        tokio::time::sleep(calculate_exponential_sleep_ms(retries, 2, minimum))
                            .await;
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

fn calculate_exponential_sleep_ms(retry_no: u32, base: u32, minimum: u32) -> Duration {
    let sleep_micros = base.pow(retry_no).saturating_mul(minimum);
    let mut rng = rand::thread_rng();
    // "full" jitter, to prevent "thundering herd". On average this still gives
    // us an exponential distribution
    let jittered_micros = rng.gen_range(minimum..=sleep_micros);

    Duration::from_micros(jittered_micros.into())
}
