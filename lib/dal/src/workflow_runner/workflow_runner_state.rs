//! This module contains [`WorkflowRunnerState`], which provides a terminating state (including
//! a [`status`](WorkflowRunnerStatus)) for the execution of a
//! [`WorkflowRunner`](crate::workflow_runner::WorkflowRunner).

use serde::{Deserialize, Serialize};

use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;

use crate::standard_model::option_object_from_row;
use crate::workflow_runner::WorkflowRunnerResult;
use crate::{impl_standard_model, pk, standard_model, Tenancy, Timestamp, Visibility};
use crate::{DalContext, WorkflowRunnerId};

const FIND_FOR_WORKFLOW_RUNNER: &str =
    include_str!("../queries/workflow_runner_state_find_for_workflow_runner.sql");

/// The [`WorkflowRunnerStatus`] represents a _terminating_ state for a
/// [`WorkflowRunner`](crate::workflow_runner::WorkflowRunner).
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WorkflowRunnerStatus {
    /// All steps executed via [`WorkflowRunner::run()`](crate::workflow_runner::WorkflowRunner::run())
    /// were successful.
    Success,
    /// At least one step executed via [`WorkflowRunner::run()`](crate::workflow_runner::WorkflowRunner::run())
    /// was not successful.
    Failure,
    /// At least one step executed via [`WorkflowRunner::run()`](crate::workflow_runner::WorkflowRunner::run())
    /// is still in progress.
    Running,
    /// No steps have been executed yet (currently, performed by executing
    /// [`WorkflowRunner::run()`](crate::workflow_runner::WorkflowRunner::run())).
    Created,
}

pk!(WorkflowRunnerStatePk);
pk!(WorkflowRunnerStateId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunnerState {
    pk: WorkflowRunnerStatePk,
    id: WorkflowRunnerStateId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,

    /// The [`WorkflowRunnerId`](crate::workflow_runner::WorkflowRunner) that this state is for.
    workflow_runner_id: WorkflowRunnerId,
    /// The status representing the [`WorkflowRunnerState`].
    status: WorkflowRunnerStatus,
    /// The execution ID of the function ran in `cyclone`. Currently, this is only populated
    /// for failures ([`WorkflowRunnerStatus::Failure`](WorkflowRunnerStatus::Failure)).
    execution_id: Option<String>,
    /// The error kind, if the status is [`WorkflowRunnerStatus::Failure`](WorkflowRunnerStatus::Failure).
    error_kind: Option<String>,
    /// The error message, if the status is [`WorkflowRunnerStatus::Failure`](WorkflowRunnerStatus::Failure).
    error_message: Option<String>,
}

impl_standard_model! {
    model: WorkflowRunnerState,
    pk: WorkflowRunnerStatePk,
    id: WorkflowRunnerStateId,
    table_name: "workflow_runner_states",
    history_event_label_base: "workflow_runner_state",
    history_event_message_name: "Workflow Runner State"
}

impl WorkflowRunnerState {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        workflow_runner_id: WorkflowRunnerId,
        status: WorkflowRunnerStatus,
        execution_id: Option<String>,
        error_kind: Option<String>,
        error_message: Option<String>,
    ) -> WorkflowRunnerResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM workflow_runner_state_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &workflow_runner_id,
                    &status.as_ref(),
                    &execution_id,
                    &error_kind,
                    &error_message,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    /// Find [`Self`] for a given [`WorkflowRunnerId`](crate::workflow_runner::WorkflowRunner).
    pub async fn find_for_workflow_runner(
        ctx: &DalContext,
        workflow_runner_id: WorkflowRunnerId,
    ) -> WorkflowRunnerResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_FOR_WORKFLOW_RUNNER,
                &[ctx.tenancy(), ctx.visibility(), &workflow_runner_id],
            )
            .await?;
        let object: Option<Self> = option_object_from_row(row)?;
        Ok(object)
    }

    pub fn workflow_runner_id(&self) -> WorkflowRunnerId {
        self.workflow_runner_id
    }

    pub fn status(&self) -> WorkflowRunnerStatus {
        self.status
    }

    pub fn execution_id(&self) -> Option<&str> {
        self.execution_id.as_deref()
    }

    pub fn error_kind(&self) -> Option<&str> {
        self.error_kind.as_deref()
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}
