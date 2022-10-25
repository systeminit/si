//! This module contains [`FixExecutionBatch`], which groups [`FixExecutions`](crate::FixExecution)
//! and indicates whether or not all "fixes" in the group have completed executing.

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::fix::execution::FixExecutionResult;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_has_many,
    DalContext, FixExecution, StandardModel, Timestamp, Visibility, WriteTenancy,
};

pk!(FixExecutionBatchPk);
pk!(FixExecutionBatchId);

/// A batch of [`FixExecutions`](crate::FixExecution). Every [`FixExecution`](crate::FixExecution)
/// must belong at one and only one [`batch`](Self).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FixExecutionBatch {
    pk: FixExecutionBatchPk,
    id: FixExecutionBatchId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    /// Indicates whether or not all "fixes" have completed execution.
    completed: bool,
}

impl_standard_model! {
    model: FixExecutionBatch,
    pk: FixExecutionBatchPk,
    id: FixExecutionBatchId,
    table_name: "fix_execution_batches",
    history_event_label_base: "fix_execution_batch",
    history_event_message_name: "FixExecutionBatch"
}

impl FixExecutionBatch {
    #[instrument(skip_all)]
    pub async fn new(ctx: &DalContext) -> FixExecutionResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM fix_execution_batch_create_v1($1, $2)",
                &[ctx.write_tenancy(), ctx.visibility()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_has_many!(
        lookup_fn: fix_executions,
        table: "fix_execution_belongs_to_fix_execution_batches",
        model_table: "fix_executions",
        returns: FixExecution,
        result: FixExecutionResult,
    );

    standard_model_accessor!(completed, bool, FixExecutionResult);
}
