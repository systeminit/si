use crate::{Func, Tenancy, TransactionsError};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::OutputStream;

use crate::func::FuncMetadataView;
use crate::{
    func::binding::FuncBindingId,
    func::execution::{FuncExecution, FuncExecutionError, FuncExecutionPk},
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    DalContext, FuncId, HistoryEventError, StandardModel, StandardModelError, Timestamp,
    Visibility,
};

use super::FuncError;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncBindingReturnValueError {
    #[error("Func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(String),
    #[error("function execution error: {0}")]
    FuncExecution(#[from] FuncExecutionError),
    #[error("func not found by id: {0}")]
    FuncNotFound(FuncId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("missing func binding return value")]
    Missing,
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("not found: {0}")]
    NotFound(FuncBindingReturnValueId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type FuncBindingReturnValueResult<T> = Result<T, FuncBindingReturnValueError>;

pk!(FuncBindingReturnValuePk);
pk!(FuncBindingReturnValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncBindingReturnValue {
    pk: FuncBindingReturnValuePk,
    id: FuncBindingReturnValueId,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior
    /// This is useful when a function binding result is used as a generator for other
    /// results - it lets us see where things came from.
    unprocessed_value: Option<serde_json::Value>,
    /// The processed return value.
    value: Option<serde_json::Value>,
    /// A record of the [`Func`](crate::Func) at the time that [`self`](Self) was
    /// created.
    func_id: FuncId,
    /// A record of the [`FuncBinding`](crate::FuncBinding) at the time that [`self`](Self) was
    /// created.
    func_binding_id: FuncBindingId,
    /// Function Execution IDs can be attached later for lookup and are optional.
    func_execution_pk: FuncExecutionPk,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: FuncBindingReturnValue,
    pk: FuncBindingReturnValuePk,
    id: FuncBindingReturnValueId,
    table_name: "func_binding_return_values",
    history_event_label_base: "function_binding_return_value",
    history_event_message_name: "Function Binding Return Value"
}

impl FuncBindingReturnValue {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        unprocessed_value: Option<serde_json::Value>,
        value: Option<serde_json::Value>,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        func_execution_pk: FuncExecutionPk,
    ) -> FuncBindingReturnValueResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_binding_return_value_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &unprocessed_value,
                    &value,
                    &func_id,
                    &func_binding_id,
                    &func_execution_pk,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    standard_model_accessor!(
        func_execution_pk,
        Pk(FuncExecutionPk),
        FuncBindingReturnValueResult
    );
    standard_model_accessor!(
        unprocessed_value,
        OptionJson<JsonValue>,
        FuncBindingReturnValueResult
    );
    standard_model_accessor!(value, OptionJson<JsonValue>, FuncBindingReturnValueResult);
    standard_model_accessor_ro!(func_id, FuncId);

    pub async fn get_output_stream(
        &self,
        ctx: &DalContext,
    ) -> FuncBindingReturnValueResult<Option<Vec<OutputStream>>> {
        if self.func_execution_pk == FuncExecutionPk::NONE {
            return Ok(None);
        }

        let func_execution = FuncExecution::get_by_pk(ctx, &self.func_execution_pk).await?;
        Ok(func_execution.into_output_stream())
    }

    /// Attempts to retrieve [`Self`] by [`FuncBindingId`].
    pub async fn get_by_func_binding_id(
        ctx: &DalContext,
        func_binding_id: FuncBindingId,
    ) -> FuncBindingReturnValueResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT fbrv FROM func_binding_return_value_get_by_func_binding_id_v1($1, $2, $3)",
                &[ctx.tenancy(), ctx.visibility(), &func_binding_id],
            )
            .await?;
        if let Some(row) = row {
            if let Some(json) = row.try_get("fbrv")? {
                return Ok(Some(serde_json::from_value(json)?));
            }
        }
        Ok(None)
    }

    /// Returns the [`FuncMetadataView`](crate::func::FuncMetadataView) based on the
    /// [`FuncId`](crate::Func) used at the creation time of [`self`](Self).
    pub async fn func_metadata_view(
        &self,
        ctx: &DalContext,
    ) -> FuncBindingReturnValueResult<FuncMetadataView> {
        let func = Func::get_by_id_or_error(ctx, self.func_id).await?;
        Ok(func.metadata_view())
    }
}
