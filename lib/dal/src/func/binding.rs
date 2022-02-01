use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech::{Client, QualificationCheckResultSuccess, ResourceSyncResultSuccess};

use crate::func::backend::boolean::{FuncBackendBoolean, FuncBackendBooleanArgs};
use crate::func::backend::integer::{FuncBackendInteger, FuncBackendIntegerArgs};
use crate::func::backend::prop_object::{FuncBackendPropObject, FuncBackendPropObjectArgs};
use crate::func::backend::{
    js_qualification::FuncBackendJsQualification,
    js_qualification::FuncBackendJsQualificationArgs,
    js_resource::FuncBackendJsResourceSync,
    js_resource::FuncBackendJsResourceSyncArgs,
    js_string::FuncBackendJsString,
    string::FuncBackendString,
    string::FuncBackendStringArgs,
    validation::{FuncBackendValidateStringValue, FuncBackendValidateStringValueArgs},
};
use crate::{
    impl_standard_model, pk, qualification::QualificationResult, standard_model,
    standard_model_accessor, standard_model_belongs_to, Func, FuncBackendError, FuncBackendKind,
    HistoryActor, HistoryEvent, HistoryEventError, StandardModel, StandardModelError, Tenancy,
    Timestamp, Visibility,
};

use super::{
    binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    execution::{FuncExecution, FuncExecutionError},
    FuncId,
};

#[derive(Error, Debug)]
pub enum FuncBindingError {
    #[error("unable to retrieve func for func binding: {0:?}")]
    FuncNotFound(FuncBindingPk),
    #[error("unable to retrieve func for func binding: {0:?}")]
    JsFuncNotFound(FuncBindingPk),
    #[error("func backend error: {0}")]
    FuncBackend(#[from] FuncBackendError),
    #[error("func backend return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
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
    #[error("func execution tracking error: {0}")]
    FuncExecutionError(#[from] FuncExecutionError),
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

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats))]
    pub async fn find_or_create(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        args: serde_json::Value,
        func_id: FuncId,
        backend_kind: FuncBackendKind,
    ) -> FuncBindingResult<(Self, bool)> {
        let row = txn
            .query_one(
                "SELECT object, created FROM func_binding_find_or_create_v1($1, $2, $3, $4)",
                &[&tenancy, &visibility, &args, &backend_kind.as_ref()],
            )
            .await?;
        let created: bool = row.try_get("created")?;

        let json_object: serde_json::Value = row.try_get("object")?;
        let object: FuncBinding = if created {
            let _history_event = HistoryEvent::new(
                txn,
                nats,
                FuncBinding::history_event_label(vec!["create"]),
                history_actor,
                FuncBinding::history_event_message("created"),
                &serde_json::json![{ "visibility": &visibility }],
                tenancy,
            )
            .await?;
            let object: FuncBinding = serde_json::from_value(json_object)?;
            object
                .set_func(txn, nats, visibility, history_actor, &func_id)
                .await?;
            object
        } else {
            serde_json::from_value(json_object)?
        };

        Ok((object, created))
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
        veritech: Client,
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

        let func = self
            .func(txn, &visibility)
            .await?
            .ok_or(FuncBindingError::FuncNotFound(self.pk))?;

        let mut execution = FuncExecution::new(txn, nats, &tenancy, &func, self).await?;

        let return_value = match self.backend_kind() {
            FuncBackendKind::Boolean => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendBooleanArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendBoolean::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::Integer => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendIntegerArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendInteger::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::JsQualification => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Dispatch)
                    .await?;

                let (tx, rx) = mpsc::channel(64);

                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;

                let handler = func
                    .handler()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let code_base64 = func
                    .code_base64()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;

                let backend_args: FuncBackendJsQualificationArgs =
                    serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendJsQualification::new(
                    veritech,
                    tx,
                    handler.to_owned(),
                    backend_args.component,
                    code_base64.to_owned(),
                )
                .execute()
                .await?;

                let veritech_result = QualificationCheckResultSuccess::deserialize(&return_value)?;
                let qual_result = QualificationResult {
                    success: veritech_result.qualified,
                    title: veritech_result.title,
                    link: veritech_result.link,
                    sub_checks: veritech_result.sub_checks,
                };

                execution.process_output(txn, nats, rx).await?;
                Some(serde_json::to_value(&qual_result)?)
            }
            FuncBackendKind::JsResourceSync => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Dispatch)
                    .await?;

                let (tx, rx) = mpsc::channel(64);

                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;

                let handler = func
                    .handler()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let code_base64 = func
                    .code_base64()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;

                let backend_args: FuncBackendJsResourceSyncArgs =
                    serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendJsResourceSync::new(
                    veritech,
                    tx,
                    handler.to_owned(),
                    backend_args.component,
                    code_base64.to_owned(),
                )
                .execute()
                .await?;

                let veritech_result = ResourceSyncResultSuccess::deserialize(&return_value)?;
                execution.process_output(txn, nats, rx).await?;
                Some(serde_json::to_value(&veritech_result)?)
            }
            FuncBackendKind::JsString => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Dispatch)
                    .await?;

                let (tx, rx) = mpsc::channel(64);
                let handler = func
                    .handler()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let code_base64 = func
                    .code_base64()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let mut map: HashMap<String, serde_json::Value> = HashMap::new();
                map.insert("value".to_owned(), self.args.clone());

                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;

                let return_value = FuncBackendJsString::new(
                    veritech,
                    tx,
                    handler.to_owned(),
                    map,
                    code_base64.to_owned(),
                )
                .execute()
                .await?;
                execution.process_output(txn, nats, rx).await?;
                Some(return_value)
            }
            FuncBackendKind::PropObject => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendPropObjectArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendPropObject::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::String => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendStringArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendString::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::Unset => None,
            FuncBackendKind::ValidateStringValue => {
                execution
                    .set_state(txn, nats, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendValidateStringValueArgs =
                    serde_json::from_value(self.args.clone())?;
                Some(FuncBackendValidateStringValue::new(args).execute()?)
            }
        };

        let fbrv = FuncBindingReturnValue::upsert(
            txn,
            nats,
            &tenancy,
            &visibility,
            &history_actor,
            return_value.clone(),
            return_value,
            *func.id(),
            self.id,
            execution.pk(),
        )
        .await?;

        execution.process_return_value(txn, nats, &fbrv).await?;
        execution
            .set_state(txn, nats, super::execution::FuncExecutionState::Success)
            .await?;

        Ok(fbrv)
    }
}
