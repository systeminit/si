use crate::WriteTenancy;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, DalContext,
    HistoryEventError, StandardModel, StandardModelError, Timestamp, Visibility,
};

use self::backend::{FuncBackendKind, FuncBackendResponseType};

pub mod backend;
pub mod binding;
pub mod binding_return_value;
pub mod builtins;
pub mod execution;

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

/// A `Func` is the declaration of the existence of a function. It has a name,
/// and corresponds to a given function backend (and its associated return types).
///
/// `handler` is the name of the entry point into the code in `code_base64`.
/// For example, if we had a code block of
/// `function myValidator(actual, expected) { return true; }` in `code_base64`,
/// the `handler` value should be `myValidator`.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pk: FuncPk,
    id: FuncId,
    name: String,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    handler: Option<String>,
    code_base64: Option<String>,
    #[serde(flatten)]
    tenancy: WriteTenancy,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        backend_kind: FuncBackendKind,
        backend_response_type: FuncBackendResponseType,
    ) -> FuncResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM func_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &backend_kind.as_ref(),
                    &backend_response_type.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, FuncResult);
    standard_model_accessor!(backend_kind, Enum(FuncBackendKind), FuncResult);
    standard_model_accessor!(
        backend_response_type,
        Enum(FuncBackendResponseType),
        FuncResult
    );
    standard_model_accessor!(handler, Option<String>, FuncResult);
    standard_model_accessor!(code_base64, Option<String>, FuncResult);
}
