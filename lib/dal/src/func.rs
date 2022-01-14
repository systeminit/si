use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, HistoryActor,
    HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};

use self::backend::{FuncBackendKind, FuncBackendResponseType};

pub mod backend;
pub mod binding;
pub mod binding_return_value;
pub mod builtins;

#[derive(Error, Debug)]
pub enum FuncError {
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

pub type FuncResult<T> = Result<T, FuncError>;

pk!(FuncPk);
pk!(FuncId);

// A `Func` is the declaration of the existence of a function. It has a name,
// and corresponds to a given function backend (and its associated return types).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pk: FuncPk,
    id: FuncId,
    name: String,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Func,
    pk: FuncPk,
    id: FuncId,
    table_name: "funcs",
    history_event_label_base: "function",
    history_event_message_name: "Function"
}

impl Func {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        backend_kind: FuncBackendKind,
        backend_response_type: FuncBackendResponseType,
    ) -> FuncResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM func_create_v1($1, $2, $3, $4, $5)",
                &[
                    &tenancy,
                    &visibility,
                    &name,
                    &backend_kind.as_ref(),
                    &backend_response_type.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, FuncResult);
    standard_model_accessor!(backend_kind, Enum(FuncBackendKind), FuncResult);
    standard_model_accessor!(
        backend_response_type,
        Enum(FuncBackendResponseType),
        FuncResult
    );
}
