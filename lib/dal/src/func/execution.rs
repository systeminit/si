use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
use veritech::{FunctionResultFailure, OutputStream};

use crate::{
    pk, Func, FuncBackendKind, FuncBackendResponseType, HistoryEventError, StandardModel,
    StandardModelError, Tenancy, Timestamp,
};

use super::{
    binding::{FuncBinding, FuncBindingId},
    binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueId},
    FuncId,
};

#[derive(Error, Debug)]
pub enum FuncExecutionError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type FuncExecutionResult<T> = Result<T, FuncExecutionError>;

pk!(FuncExecutionPk);
pk!(FuncExecutionId);

// Are these the right states? -- Adam
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    strum_macros::EnumString,
    strum_macros::Display,
    Copy,
)]
pub enum FuncExecutionState {
    Create,
    Dispatch,
    Start,
    Run,
    Success,
    Failure,
}

/// FuncExecutions record that a function has executed, all the
/// various context required to understand the execution, and
/// contains the log of the output stream for the function.
///
/// It is not part of the 'standard model' - it doesn't participate
/// in change sets, and is only used for reference. Essentially the
/// func equivalent of a `HistoryEvent`
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
    #[tracing::instrument(skip(txn, nats))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        func: &Func,
        func_binding: &FuncBinding,
    ) -> FuncExecutionResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM func_execution_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &tenancy,
                    &FuncExecutionState::Start.to_string(),
                    &func.id(),
                    &func_binding.id(),
                    &func_binding.args(),
                    &func_binding.backend_kind().to_string(),
                    &func.backend_response_type().to_string(),
                    &func.handler(),
                    &func.code_base64(),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // This needs to be some kind of 'immediate mode' publish.
        nats.publish("funcExecution", &json).await?;
        let object: FuncExecution = serde_json::from_value(json)?;
        Ok(object)
    }

    pub fn state(&self) -> FuncExecutionState {
        self.state
    }

    pub async fn set_state(
        &mut self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        state: FuncExecutionState,
    ) -> FuncExecutionResult<()> {
        let row = txn
            .query_one(
                "SELECT object FROM func_execution_set_state_v1($1, $2)",
                &[&self.pk, &state.to_string()],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        // This needs to be some kind of 'immediate mode' publish.
        nats.publish("funcExecution", &json).await?;
        let mut object: FuncExecution = serde_json::from_value(json)?;
        std::mem::swap(self, &mut object);
        Ok(())
    }

    /// Takes the receiver stream from a Veritech function execution, and stores the output.
    pub async fn process_output(
        &mut self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        mut rx: Receiver<OutputStream>,
    ) -> FuncExecutionResult<()> {
        // Right now, we consume everything. This should really be happening in a separate thread altogether, and
        // persisting the output lines as they come in.  But this works for now, and should be easy enough to
        // refactor.
        let mut output = Vec::new();
        while let Some(output_stream) = rx.recv().await {
            output.push(output_stream);
        }
        self.set_output_stream(txn, nats, output).await
    }

    pub fn output_stream(&self) -> Option<&Vec<OutputStream>> {
        self.output_stream.as_ref()
    }

    pub async fn set_output_stream(
        &mut self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        output_stream: Vec<OutputStream>,
    ) -> FuncExecutionResult<()> {
        let output_stream_json = serde_json::to_value(&output_stream)?;
        let row = txn
            .query_one(
                "SELECT object FROM func_execution_set_output_stream_v1($1, $2)",
                &[&self.pk, &output_stream_json],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish("funcExecution", &json).await?;
        let mut object: FuncExecution = serde_json::from_value(json)?;
        std::mem::swap(self, &mut object);
        Ok(())
    }

    /// Take the return value of a function binding, and store its results.
    pub async fn process_return_value(
        &mut self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        func_binding_return_value: &FuncBindingReturnValue,
    ) -> FuncExecutionResult<()> {
        let row = txn
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
        nats.publish("funcExecution", &json).await?;
        let mut object: FuncExecution = serde_json::from_value(json)?;
        std::mem::swap(self, &mut object);

        Ok(())
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
}
