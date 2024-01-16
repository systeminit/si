use crate::{standard_model_accessor_ro, Tenancy, TransactionsError};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
use veritech_client::{FunctionResultFailure, OutputStream};

use crate::standard_model::object_from_row;
use crate::{
    pk, DalContext, Func, FuncBackendKind, FuncBackendResponseType, HistoryEventError,
    StandardModel, StandardModelError, Timestamp,
};

use super::{
    binding::{FuncBinding, FuncBindingId},
    binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueId},
    FuncId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncExecutionError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    NotFound(FuncExecutionPk),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type FuncExecutionResult<T> = Result<T, FuncExecutionError>;

pk!(FuncExecutionPk);

// Are these the right states? -- Adam
#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, Copy,
)]
pub enum FuncExecutionState {
    Create,
    Dispatch,
    Failure,
    Run,
    Start,
    Success,
}

/// [`FuncExecutions`](Self) record that a [`function`](crate::Func) has executed alongside all the
/// context required to understand the execution as well as the log of the output stream for the
/// [`function`](crate::Func).
///
/// It's not part of the [`standard model`](crate::standard_model) as it doesn't participate in
/// [`change sets`](crate::ChangeSet), and is only used for reference. Essentially, this is the
/// [`Func`](crate::Func) equivalent of a [`HistoryEvent`](crate::HistoryEvent).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncExecution {
    pk: FuncExecutionPk,
    state: FuncExecutionState,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    func_binding_args: serde_json::Value,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    func_binding_return_value_id: Option<FuncBindingReturnValueId>,
    handler: Option<String>,
    code_base64: Option<String>,
    unprocessed_value: Option<serde_json::Value>,
    value: Option<serde_json::Value>,
    output_stream: Option<Vec<OutputStream>>,
    function_failure: Option<FunctionResultFailure>,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl FuncExecution {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func: &Func,
        func_binding: &FuncBinding,
    ) -> FuncExecutionResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_execution_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    ctx.tenancy(),
                    &FuncExecutionState::Start.to_string(),
                    &func.id,
                    &func_binding.id(),
                    &func_binding.args(),
                    &func_binding.backend_kind().to_string(),
                    &func.backend_response_type.to_string(),
                    &func.handler.as_deref(),
                    &func.code_base64.as_deref(),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // This needs to be some kind of 'immediate mode' publish.
        ctx.txns()
            .await?
            .nats()
            .publish("funcExecution", &json)
            .await?;
        let object: FuncExecution = serde_json::from_value(json)?;
        Ok(object)
    }

    pub fn state(&self) -> FuncExecutionState {
        self.state
    }

    pub async fn set_state(
        &mut self,
        ctx: &DalContext,
        state: FuncExecutionState,
    ) -> FuncExecutionResult<()> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_execution_set_state_v1($1, $2)",
                &[&self.pk, &state.to_string()],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // This needs to be some kind of 'immediate mode' publish.
        ctx.txns()
            .await?
            .nats()
            .publish("funcExecution", &json)
            .await?;
        let mut object: FuncExecution = serde_json::from_value(json)?;
        std::mem::swap(self, &mut object);
        Ok(())
    }

    /// Takes the receiver stream from a Veritech function execution, and stores the output.
    pub async fn process_output(
        &mut self,
        ctx: &DalContext,
        mut rx: Receiver<OutputStream>,
    ) -> FuncExecutionResult<()> {
        // Right now, we consume everything. This should really be happening in a separate thread altogether, and
        // persisting the output lines as they come in.  But this works for now, and should be easy enough to
        // refactor.
        let mut output = Vec::new();
        while let Some(output_stream) = rx.recv().await {
            output.push(output_stream);
        }
        self.set_output_stream(ctx, output).await
    }

    pub fn output_stream(&self) -> Option<&Vec<OutputStream>> {
        self.output_stream.as_ref()
    }

    pub fn into_output_stream(self) -> Option<Vec<OutputStream>> {
        self.output_stream
    }

    pub async fn set_output_stream(
        &mut self,
        ctx: &DalContext,
        output_stream: Vec<OutputStream>,
    ) -> FuncExecutionResult<()> {
        let output_stream_json = serde_json::to_value(&output_stream)?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_execution_set_output_stream_v1($1, $2)",
                &[&self.pk, &output_stream_json],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        ctx.txns()
            .await?
            .nats()
            .publish("funcExecution", &json)
            .await?;
        let mut object: FuncExecution = serde_json::from_value(json)?;
        std::mem::swap(self, &mut object);
        Ok(())
    }

    /// Take the return value of a function binding, and store its results.
    pub async fn process_return_value(
        &mut self,
        ctx: &DalContext,
        func_binding_return_value: &FuncBindingReturnValue,
    ) -> FuncExecutionResult<()> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_execution_set_return_value_v1($1, $2, $3, $4)",
                &[
                    &self.pk,
                    &func_binding_return_value.id(),
                    &func_binding_return_value.value(),
                    &func_binding_return_value.unprocessed_value(),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        ctx.txns()
            .await?
            .nats()
            .publish("funcExecution", &json)
            .await?;
        let mut object: FuncExecution = serde_json::from_value(json)?;
        std::mem::swap(self, &mut object);

        Ok(())
    }

    pub fn pk(&self) -> FuncExecutionPk {
        self.pk
    }

    #[instrument(skip(ctx))]
    pub async fn get_by_pk(ctx: &DalContext, pk: &FuncExecutionPk) -> FuncExecutionResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM get_by_pk_v1($1, $2)",
                &[&"func_executions", &pk],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_latest_execution_by_func_id(
        ctx: &DalContext,
        func_id: &FuncId,
    ) -> FuncExecutionResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT row_to_json(fe.*) as object FROM func_executions fe WHERE func_id = $1 ORDER BY updated_at LIMIT 1",
                &[func_id],
            )
            .await?;
        Ok(object_from_row(row)?)
    }

    pub fn func_binding_return_value_id(&self) -> Option<FuncBindingReturnValueId> {
        self.func_binding_return_value_id
    }

    pub fn value(&self) -> Option<&serde_json::Value> {
        self.value.as_ref()
    }

    pub fn unprocessed_value(&self) -> Option<&serde_json::Value> {
        self.unprocessed_value.as_ref()
    }

    standard_model_accessor_ro!(func_id, FuncId);
    standard_model_accessor_ro!(function_failure, Option<FunctionResultFailure>);
    standard_model_accessor_ro!(func_binding_args, serde_json::Value);
    standard_model_accessor_ro!(handler, Option<String>);
    standard_model_accessor_ro!(backend_kind, FuncBackendKind);
    standard_model_accessor_ro!(backend_response_type, FuncBackendResponseType);
    standard_model_accessor_ro!(code_base64, Option<String>);
}
