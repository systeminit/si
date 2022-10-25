use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::fix::execution::batch::FixExecutionBatchId;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::workflow_runner::workflow_runner_state::{WorkflowRunnerState, WorkflowRunnerStateId};
use crate::{
    impl_standard_model, pk, standard_model, standard_model_belongs_to, ComponentId,
    ConfirmationResolverId, DalContext, HistoryEventError, StandardModel, StandardModelError,
    Timestamp, Visibility, WorkflowPrototypeId, WorkflowRunnerError, WriteTenancy,
};
use crate::{FixExecutionBatch, WorkflowRunner};

pub mod batch;

#[derive(Error, Debug)]
pub enum FixExecutionError {
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

    #[error("cannot set batch for {0}: fix execution batch ({1}) already marked as completed")]
    BatchAlreadyComplete(FixExecutionId, FixExecutionBatchId),
    #[error("fix execution batch not found for id: {0}")]
    MissingFixExecutionBatch(FixExecutionBatchId),
}

pub type FixExecutionResult<T> = Result<T, FixExecutionError>;

pk!(FixExecutionPk);
pk!(FixExecutionId);

/// A record of a "fix" after it has been executed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FixExecution {
    pk: FixExecutionPk,
    id: FixExecutionId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    confirmation_resolver_id: ConfirmationResolverId,
    workflow_runner_state_id: WorkflowRunnerStateId,
    logs: Vec<String>,
}

impl_standard_model! {
    model: FixExecution,
    pk: FixExecutionPk,
    id: FixExecutionId,
    table_name: "fix_executions",
    history_event_label_base: "fix_execution",
    history_event_message_name: "FixExecution"
}

impl FixExecution {
    /// Create [`Self`] and ensure it belongs to a [`FixExecutionBatch`](crate::FixExecutionBatch)
    /// since every [`execution`](Self) must belong to a [`batch`](crate::FixExecutionBatch).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        fix_execution_batch_id: FixExecutionBatchId,
        confirmation_resolver_id: ConfirmationResolverId,
        workflow_runner_state_id: WorkflowRunnerStateId,
        logs: Vec<String>,
    ) -> FixExecutionResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM fix_execution_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &confirmation_resolver_id,
                    &workflow_runner_state_id,
                    &logs,
                ],
            )
            .await?;
        let object: FixExecution = standard_model::finish_create_from_row(ctx, row).await?;
        object
            .set_fix_execution_batch(ctx, fix_execution_batch_id)
            .await?;
        Ok(object)
    }

    /// While [`Self::new()`] creates a new row using data from an execution that's already occurred,
    /// this method performs the "fix" and _then_ calls [`Self::new()`].
    pub async fn new_and_perform_fix(
        ctx: &DalContext,
        batch_id: FixExecutionBatchId,
        confirmation_resolver_id: ConfirmationResolverId,
        run_id: usize,
        action_workflow_prototype_id: WorkflowPrototypeId,
        component_id: ComponentId,
    ) -> FixExecutionResult<(Self, WorkflowRunnerState)> {
        let (
            _runner,
            runner_state,
            func_binding_return_values,
            _created_resources,
            _updated_resources,
        ) = WorkflowRunner::run(ctx, run_id, action_workflow_prototype_id, component_id).await?;

        let mut logs = Vec::new();
        for func_binding_return_value in func_binding_return_values {
            for stream in func_binding_return_value
                .get_output_stream(ctx)
                .await?
                .unwrap_or_default()
            {
                match stream.data {
                    Some(data) => logs.push((
                        stream.timestamp,
                        format!(
                            "{} {}",
                            stream.message,
                            serde_json::to_string_pretty(&data)?
                        ),
                    )),
                    None => logs.push((stream.timestamp, stream.message)),
                }
            }
        }
        logs.sort_by_key(|(timestamp, _)| *timestamp);
        let logs = logs.into_iter().map(|(_, log)| log).collect();

        let execution = Self::new(
            ctx,
            batch_id,
            confirmation_resolver_id,
            *runner_state.id(),
            logs,
        )
        .await?;
        Ok((execution, runner_state))
    }

    /// A wrapper around the standard model function in order to ensure that the
    /// [`batch`](crate::FixExecutionBatch) is not in a "completed" state.
    pub async fn set_fix_execution_batch(
        &self,
        ctx: &DalContext,
        batch_id: FixExecutionBatchId,
    ) -> FixExecutionResult<()> {
        let batch = FixExecutionBatch::get_by_id(ctx, &batch_id)
            .await?
            .ok_or(FixExecutionError::MissingFixExecutionBatch(batch_id))?;
        if batch.completed() {
            return Err(FixExecutionError::BatchAlreadyComplete(self.id, batch_id));
        }
        self.set_fix_execution_batch_unchecked(ctx, &batch_id)
            .await?;
        Ok(())
    }

    standard_model_belongs_to!(
        lookup_fn: fix_execution_batch,
        set_fn: set_fix_execution_batch_unchecked,
        unset_fn: unset_fix_execution_batch,
        table: "fix_execution_belongs_to_fix_execution_batch",
        model_table: "fix_execution_batches",
        belongs_to_id: FixExecutionBatchId,
        returns: FixExecutionBatch,
        result: FixExecutionResult,
    );

    pub fn logs(&self) -> Vec<String> {
        self.logs.clone()
    }
}
