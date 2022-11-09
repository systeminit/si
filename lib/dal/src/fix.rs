//! This module contains the concept of "fixes".

use chrono::Utc;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::fix::batch::FixBatchId;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    ComponentError, ComponentId, ConfirmationPrototypeId, ConfirmationResolverError,
    ConfirmationResolverId, ConfirmationResolverTreeError, DalContext, FixResolverError,
    HistoryEventError, StandardModel, StandardModelError, Timestamp, Visibility,
    WorkflowPrototypeId, WorkflowRunnerError, WorkflowRunnerStatus, WriteTenancy, WsEvent,
    WsPayload,
};
use crate::{FixBatch, WorkflowRunner};

/// Contains the ability to group fixes together.
pub mod batch;
/// Contains recommendations that are used to create and run fixes.
pub mod recommendation;
/// Contains the ability to resolve _current_ fixes, provided by [`FixResolver`](crate::FixResolver).
pub mod resolver;

/// The completion status of a [`Fix`] or [`FixBatch`](crate::FixBatch).
#[derive(
    Deserialize,
    Serialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ToSql,
    FromSql,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum FixCompletionStatus {
    /// The [`Fix`] or at least one [`Fix`] in the [`FixBatch`](crate::FixBatch) executed with
    /// error(s).
    Error,
    /// The [`Fix`] or at least one [`Fix`] in the [`FixBatch`](crate::FixBatch) was or may have
    /// been executed without error, but it or they were unsuccessful during execution.
    Failure,
    /// The [`Fix`] or all [`Fixes`](Fix) in the [`FixBatch`](crate::FixBatch) were executed and
    /// successful.
    Success,
}

impl FixCompletionStatus {
    /// Attempt to convert a [`WorkflowRunnerStatus`](crate::WorkflowRunnerStatus) to [`Self`].
    pub fn from_workflow_runner_status(status: WorkflowRunnerStatus) -> FixResult<Self> {
        if let WorkflowRunnerStatus::Success = status {
            Ok(Self::Success)
        } else if let WorkflowRunnerStatus::Failure = status {
            Ok(Self::Failure)
        } else {
            Err(FixError::IncompatibleWorkflowRunnerStatus(status))
        }
    }
}

#[derive(Error, Debug)]
pub enum FixError {
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    ConfirmationResolver(#[from] ConfirmationResolverError),
    #[error(transparent)]
    ConfirmationResolverTree(#[from] ConfirmationResolverTreeError),
    #[error(transparent)]
    FixResolver(#[from] FixResolverError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    WorkflowRunner(#[from] WorkflowRunnerError),

    #[error("cannot stamp batch or fix as started since it already finished")]
    AlreadyFinished,
    #[error("cannot stamp batch or fix as started since it already started")]
    AlreadyStarted,
    #[error("cannot set batch for {0}: fix batch ({1}) already finished")]
    BatchAlreadyFinished(FixId, FixBatchId),
    #[error("cannot set batch for {0}: fix batch ({1}) already started")]
    BatchAlreadyStarted(FixId, FixBatchId),
    #[error("confirmation prototype {0} not found")]
    ConfirmationPrototypeNotFound(ConfirmationPrototypeId),
    #[error("completion status is empty")]
    EmptyCompletionStatus,
    #[error("workflow runner status {0} cannot be converted to fix completion status")]
    IncompatibleWorkflowRunnerStatus(WorkflowRunnerStatus),
    #[error("fix not found for id: {0}")]
    MissingFix(FixId),
    #[error("fix batch not found for id: {0}")]
    MissingFixBatch(FixBatchId),
    #[error("no fixes in batch: fix batch is empty")]
    NoFixesInBatch(FixBatchId),
    #[error("cannot stamp batch or fix as finished since it has not yet been started")]
    NotYetStarted,
}

pub type FixResult<T> = Result<T, FixError>;

pk!(FixPk);
pk!(FixId);

/// A record of a "fix" after it has been executed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pk: FixPk,
    id: FixId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    /// The [`ConfirmationResolver`](crate::ConfirmationResolver) used to determine which fix to run.
    confirmation_resolver_id: ConfirmationResolverId,
    /// The [`Component`](crate::Component) being fixed.
    component_id: ComponentId,
    // TODO(nick): convert to Vec<String> once it works with standard model accessor.
    /// The logs generated during the fix.
    logs: Option<String>,
    /// The name of the [`ActionPrototype`](crate::action_prototype::ActionPrototype) used.
    action: Option<String>,

    // TODO(nick): convert to Option<DateTime<Utc>> once standard model accessor can accommodate both
    // Option<T<U>> and can handle "timestamp with time zone <--> DateTime<Utc>".
    /// Indicates when the [`Fix`] started execution when populated.
    started_at: Option<String>,
    // TODO(nick): convert to Option<DateTime<Utc>> once standard model accessor can accommodate both
    // Option<T<U>> and can handle "timestamp with time zone <--> DateTime<Utc>".
    /// Indicates when the [`Fix`] finished execution when populated.
    finished_at: Option<String>,
    /// Indicates the state of the [`Fix`] when finished.
    completion_status: Option<FixCompletionStatus>,

    /// Contains a message related to the completion.
    completion_message: Option<String>,
}

impl_standard_model! {
    model: Fix,
    pk: FixPk,
    id: FixId,
    table_name: "fixes",
    history_event_label_base: "fix",
    history_event_message_name: "Fix"
}

impl Fix {
    /// Create [`Self`] and ensure it belongs to a [`FixBatch`](crate::FixBatch)
    /// since every [`fix`](Self) must belong to a [`batch`](crate::FixBatch).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        fix_batch_id: FixBatchId,
        confirmation_resolver_id: ConfirmationResolverId,
        component_id: ComponentId,
    ) -> FixResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM fix_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &confirmation_resolver_id,
                    &component_id,
                ],
            )
            .await?;
        let object: Fix = standard_model::finish_create_from_row(ctx, row).await?;
        object.set_fix_batch(ctx, fix_batch_id).await?;
        Ok(object)
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    standard_model_accessor!(logs, Option<String>, FixResult);
    standard_model_accessor!(action, Option<String>, FixResult);
    standard_model_accessor!(started_at, Option<String>, FixResult);
    standard_model_accessor!(finished_at, Option<String>, FixResult);
    standard_model_accessor!(
        completion_status,
        Option<Enum(FixCompletionStatus)>,
        FixResult
    );
    standard_model_accessor!(completion_message, Option<String>, FixResult);

    standard_model_belongs_to!(
        lookup_fn: fix_batch,
        set_fn: set_fix_batch_unchecked,
        unset_fn: unset_fix_batch,
        table: "fix_belongs_to_fix_batch",
        model_table: "fix_batches",
        belongs_to_id: FixBatchId,
        returns: FixBatch,
        result: FixResult,
    );

    /// A wrapper around the standard model function in order to ensure that the
    /// [`batch`](crate::FixBatch) is has not yet been executed.
    pub async fn set_fix_batch(&self, ctx: &DalContext, batch_id: FixBatchId) -> FixResult<()> {
        let batch = FixBatch::get_by_id(ctx, &batch_id)
            .await?
            .ok_or(FixError::MissingFixBatch(batch_id))?;
        if batch.started_at().is_some() {
            return Err(FixError::BatchAlreadyStarted(self.id, batch_id));
        }
        if batch.finished_at().is_some() {
            return Err(FixError::BatchAlreadyFinished(self.id, batch_id));
        }
        self.set_fix_batch_unchecked(ctx, &batch_id).await?;
        Ok(())
    }

    /// Executes the [`fix`](Self). Returns true if some resource got updated, false if not
    pub async fn run(
        &mut self,
        ctx: &DalContext,
        run_id: usize,
        action_workflow_prototype_id: WorkflowPrototypeId,
        action_name: String,
    ) -> FixResult<bool> {
        // Stamp started and run the workflow.
        self.stamp_started(ctx).await?;
        self.set_action(ctx, Some(action_name)).await?;
        let runner_result =
            WorkflowRunner::run(ctx, run_id, action_workflow_prototype_id, self.component_id).await;

        // Evaluate the workflow result.
        match runner_result {
            Ok(post_run_data) => {
                let (
                    _runner,
                    runner_state,
                    func_binding_return_values,
                    created_resources,
                    updated_resources,
                ) = post_run_data;

                // Set the run as completed. Record the error message if it exists.
                let completion_status =
                    FixCompletionStatus::from_workflow_runner_status(runner_state.status())?;
                self.stamp_finished(
                    ctx,
                    completion_status,
                    runner_state.error_message().map(|s| s.to_string()),
                )
                .await?;

                // Gather and store the logs.
                let mut logs = Vec::new();
                for func_binding_return_value in func_binding_return_values {
                    for stream in func_binding_return_value
                        .get_output_stream(ctx)
                        .await?
                        .unwrap_or_default()
                    {
                        logs.push((stream.timestamp, stream.message));
                    }
                }
                logs.sort_by_key(|(timestamp, _)| *timestamp);
                let logs: Vec<String> = logs.into_iter().map(|(_, log)| log).collect();

                // TODO(nick): change once logs' type is converted from Option<String> to Vec<String>.
                self.set_logs(ctx, Some(logs.join("\n"))).await?;
                Ok(!updated_resources.is_empty() || !created_resources.is_empty())
            }
            Err(e) => {
                // If the workflow had an error, we can record an error completion status with
                // the error as a message.
                self.stamp_finished(ctx, FixCompletionStatus::Error, Some(format!("{:?}", e)))
                    .await?;
                Ok(false)
            }
        }
    }

    /// A safe wrapper around setting completion-related columns.
    pub async fn stamp_finished(
        &mut self,
        ctx: &DalContext,
        completion_status: FixCompletionStatus,
        completion_message: Option<String>,
    ) -> FixResult<()> {
        if self.started_at.is_some() {
            self.set_finished_at(ctx, Some(format!("{}", Utc::now())))
                .await?;
            self.set_completion_status(ctx, Some(completion_status))
                .await?;
            if completion_message.is_some() {
                self.set_completion_message(ctx, completion_message).await?;
            }
            Ok(())
        } else {
            Err(FixError::NotYetStarted)
        }
    }

    /// A safe wrapper around setting the started column.
    pub async fn stamp_started(&mut self, ctx: &DalContext) -> FixResult<()> {
        if self.started_at.is_some() {
            Err(FixError::AlreadyStarted)
        } else if self.finished_at.is_some() {
            Err(FixError::AlreadyFinished)
        } else {
            self.set_started_at(ctx, Some(format!("{}", Utc::now())))
                .await?;
            Ok(())
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixReturn {
    id: FixId,
    batch_id: FixBatchId,
    confirmation_resolver_id: ConfirmationResolverId,
    action: String,
    completion_status: FixCompletionStatus,
    output: Vec<String>,
}

impl WsEvent {
    pub fn fix_return(
        ctx: &DalContext,
        id: FixId,
        batch_id: FixBatchId,
        confirmation_resolver_id: ConfirmationResolverId,
        action: String,
        completion_status: FixCompletionStatus,
        output: Vec<String>,
    ) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::FixReturn(FixReturn {
                id,
                batch_id,
                confirmation_resolver_id,
                action,
                completion_status,
                output,
            }),
        )
    }
}
