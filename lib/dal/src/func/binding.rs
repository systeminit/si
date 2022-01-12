use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use serde_json::Value as JsonValue;

use crate::{
    func::backend::{FuncBackendString, FuncBackendStringArgs},
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    Func, FuncBackendError, FuncBackendKind, HistoryActor, HistoryEventError, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility,
};

use super::{
    binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    FuncId,
};

#[derive(Error, Debug)]
pub enum FuncBindingError {
    #[error("unable to retrieve func for func binding: {0:?}")]
    FuncNotFound(FuncBindingPk),
    #[error("func backend error: {0}")]
    FuncBackend(#[from] FuncBackendError),
    #[error("func backend return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("incorrect return value type, expected: {0}")]
    IncorrectReturnValueType(&'static str),
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

pub type FuncBindingResult<T> = Result<T, FuncBindingError>;

// A `FuncBinding` binds an execution context to a `Func`, so that it can be
// executed. So for example, you would create a `FuncBinding` with the arguments
// to the Func, and then say that this binding `belongs_to` a `prop`, or a `schema`,
// etc.
pk!(FuncBindingPk);
pk!(FuncBindingId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncBinding {
    pk: FuncBindingPk,
    id: FuncBindingId,
    args: serde_json::Value,
    backend_kind: FuncBackendKind,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: FuncBinding,
    pk: FuncBindingPk,
    id: FuncBindingId,
    table_name: "func_bindings",
    history_event_label_base: "function_binding",
    history_event_message_name: "Function Binding"
}

impl FuncBinding {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        args: serde_json::Value,
        func_id: FuncId,
        backend_kind: FuncBackendKind,
    ) -> FuncBindingResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM func_binding_create_v1($1, $2, $3, $4)",
                &[&tenancy, &visibility, &args, &backend_kind.as_ref()],
            )
            .await?;
        let object: FuncBinding = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        object
            .set_func(txn, nats, visibility, history_actor, &func_id)
            .await?;
        Ok(object)
    }

    standard_model_accessor!(args, Json<JsonValue>, FuncBindingResult);
    standard_model_accessor!(backend_kind, Enum(FuncBackendKind), FuncBindingResult);
    standard_model_belongs_to!(
        lookup_fn: func,
        set_fn: set_func,
        unset_fn: unset_func,
        table: "func_binding_belongs_to_func",
        model_table: "funcs",
        belongs_to_id: FuncId,
        returns: Func,
        result: FuncBindingResult,
    );

    pub async fn execute(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
    ) -> FuncBindingResult<FuncBindingReturnValue> {
        // NOTE: for now (subject to change), we are using the same visibility and tenancy to
        // create a return value. This seems to make sense as why would a return value be in a
        // different tenancy or have different visibility. But maybe?
        let visibility = self.visibility;
        let tenancy = self.tenancy.clone();
        // NOTE: the actor that executes this binding may not correspond to a user and that might
        // not make sense anyway. However, if it's a "caller's responsibility", then we can make
        // this an argument to this function
        let history_actor = HistoryActor::SystemInit;

        let return_value = match self.backend_kind() {
            FuncBackendKind::String => {
                let args: FuncBackendStringArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendString::new(args).execute()?;
                if !return_value.is_string() {
                    return Err(FuncBindingError::IncorrectReturnValueType("String"));
                }
                Some(return_value)
            }
        };

        let func = self
            .func(txn, &visibility)
            .await?
            .ok_or(FuncBindingError::FuncNotFound(self.pk))?;

        FuncBindingReturnValue::new(
            txn,
            nats,
            &tenancy,
            &visibility,
            &history_actor,
            return_value.clone(),
            return_value,
            *func.id(),
            self.id,
        )
        .await
        .map_err(Into::into)
    }
}
