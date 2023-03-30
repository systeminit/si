use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_nats::NatsError;
use si_data_pg::PgPoolError;
use thiserror::Error;
use tokio::task::JoinError;
use ulid::Ulid;

use crate::{
    fix::FixError, func::binding_return_value::FuncBindingReturnValueError,
    status::StatusUpdaterError, workflow_runner::WorkflowRunnerError, AccessBuilder,
    ActionPrototypeError, AttributeValueError, ComponentError, ComponentId, DalContext,
    DalContextBuilder, FixBatchId, FixResolverError, StandardModelError, TransactionsError,
    Visibility, WsEventError,
};

#[derive(Error, Debug)]
pub enum JobConsumerError {
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    Fix(#[from] FixError),
    #[error("missing fix execution batch for id: {0}")]
    MissingFixBatch(FixBatchId),
    #[error("Invalid job arguments. Expected: {0} Actual: {1:?}")]
    InvalidArguments(String, Vec<Value>),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    TokioTask(#[from] JoinError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Council(#[from] council_server::client::Error),
    #[error("Protocol error with council: {0}")]
    CouncilProtocol(String),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    WorkflowRunner(#[from] WorkflowRunnerError),
    #[error(transparent)]
    FixResolver(#[from] FixResolverError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),
    #[error("no schema found for component {0}")]
    NoSchemaFound(ComponentId),
    #[error("no schema variant found for component {0}")]
    NoSchemaVariantFound(ComponentId),
    #[error("action named {0} not found for component {1}")]
    ActionNotFound(String, ComponentId),
    #[error(transparent)]
    StatusUpdaterError(#[from] StatusUpdaterError),
    #[error("arg {0:?} not found at index {1}")]
    ArgNotFound(JobInfo, usize),
}

impl From<JobConsumerError> for std::io::Error {
    fn from(jce: JobConsumerError) -> Self {
        Self::new(std::io::ErrorKind::InvalidData, jce)
    }
}

pub type JobConsumerResult<T> = Result<T, JobConsumerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextJobInfo {
    pub job: JobInfo,
    pub wait_for_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub kind: String,
    pub queue: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub enqueued_at: Option<DateTime<Utc>>,
    pub at: Option<DateTime<Utc>>,
    pub args: Vec<Value>,
    pub retry: Option<isize>,
    pub custom: JobConsumerCustomPayload,
    pub subsequent_jobs: VecDeque<NextJobInfo>,
}

impl JobInfo {
    pub fn args(&self) -> &[Value] {
        &self.args
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }
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
                .ok_or(JobConsumerError::ArgNotFound(self.clone(), 1))
                .expect("unable to get access builder"),
        )
        .expect("unable to deserialize access builder")
    }

    fn visibility(&self) -> Visibility {
        serde_json::from_value(
            self.args
                .get(2)
                .cloned()
                .ok_or(JobConsumerError::ArgNotFound(self.clone(), 2))
                .expect("unable to get access builder"),
        )
        .expect("unable to deserialize access builder")
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JobInvocationId(Ulid);

impl JobInvocationId {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for JobInvocationId {
    fn default() -> Self {
        Self(Ulid::new())
    }
}

impl fmt::Display for JobInvocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobConsumerCustomPayload {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
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
    /// Horrible hack, exists to support sync processor, they need that all jobs run within the provided DalContext, without commiting any transactions, or writing to unrelated transactions
    /// And since it's sync the data sharing issue that appears in dependent values update running in parallel in pinga, sharing data, synchronized by council, stops existing
    fn set_sync(&mut self) {}

    /// Intended to be defined by implementations of this trait.
    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()>;

    /// Called on the trait object to set up the data necessary to run the job,
    /// and in-turn calls the `run` method. Can be overridden by an implementation
    /// of the trait if you need more control over how the `DalContext` is managed
    /// during the lifetime of the job.
    async fn run_job(&self, ctx_builder: DalContextBuilder) -> JobConsumerResult<()> {
        let ctx = ctx_builder
            .build(self.access_builder().build(self.visibility()))
            .await?;

        self.run(&ctx).await?;

        ctx.commit().await?;

        Ok(())
    }
}
