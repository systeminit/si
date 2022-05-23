use crate::WriteTenancy;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech::{
    CodeGenerationResultSuccess, OutputStream, QualificationCheckResultSuccess,
    ResourceSyncResultSuccess,
};

use crate::func::backend::array::{FuncBackendArray, FuncBackendArrayArgs};
use crate::func::backend::boolean::{FuncBackendBoolean, FuncBackendBooleanArgs};
use crate::func::backend::identity::FuncBackendIdentityArgs;
use crate::func::backend::integer::{FuncBackendInteger, FuncBackendIntegerArgs};
use crate::func::backend::map::{FuncBackendMap, FuncBackendMapArgs};
use crate::func::backend::prop_object::{FuncBackendPropObject, FuncBackendPropObjectArgs};
use crate::func::backend::{
    js_attribute::{FuncBackendJsAttribute, FuncBackendJsAttributeArgs},
    js_code_generation::FuncBackendJsCodeGeneration,
    js_code_generation::FuncBackendJsCodeGenerationArgs,
    js_qualification::FuncBackendJsQualification,
    js_qualification::FuncBackendJsQualificationArgs,
    js_resource::FuncBackendJsResourceSync,
    js_resource::FuncBackendJsResourceSyncArgs,
    string::FuncBackendString,
    string::FuncBackendStringArgs,
    validation::{FuncBackendValidateStringValue, FuncBackendValidateStringValueArgs},
};
use crate::DalContext;
use crate::{
    component::ComponentViewError, impl_standard_model, pk, qualification::QualificationResult,
    standard_model, standard_model_accessor, standard_model_belongs_to, ComponentView, Func,
    FuncBackendError, FuncBackendKind, HistoryEvent, HistoryEventError, ReadTenancyError,
    StandardModel, StandardModelError, Timestamp, Visibility,
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
    #[error("func binding not found: {0}")]
    NotFound(FuncBindingId),
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
    #[error("component view error: {0}")]
    ComponentView(#[from] ComponentViewError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("output stream error: {0}")]
    OutputStream(#[from] mpsc::error::SendError<OutputStream>),
}

pub type FuncBindingResult<T> = Result<T, FuncBindingError>;

// A `FuncBinding` binds an execution context to a `Func`, so that it can be
// executed. So for example, you would create a `FuncBinding` with the arguments
// to the Func, and then say that this binding `belongs_to` a `prop`, or a `schema`,
// etc.
pk!(FuncBindingPk);
pk!(FuncBindingId);

// NOTE(nick,jacob): the [`HashMap`] of input sockets will likely live here in the future.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncBinding {
    pk: FuncBindingPk,
    id: FuncBindingId,
    args: serde_json::Value,
    backend_kind: FuncBackendKind,
    #[serde(flatten)]
    tenancy: WriteTenancy,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        args: serde_json::Value,
        func_id: FuncId,
        backend_kind: FuncBackendKind,
    ) -> FuncBindingResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM func_binding_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &args,
                    &backend_kind.as_ref(),
                ],
            )
            .await?;
        let object: FuncBinding = standard_model::finish_create_from_row(ctx, row).await?;
        object.set_func(ctx, &func_id).await?;
        Ok(object)
    }

    #[instrument(skip_all)]
    pub async fn find_or_create(
        ctx: &DalContext<'_, '_>,
        args: serde_json::Value,
        func_id: FuncId,
        backend_kind: FuncBackendKind,
    ) -> FuncBindingResult<(Self, bool)> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object, created FROM func_binding_find_or_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &args,
                    &backend_kind.as_ref(),
                    &func_id,
                ],
            )
            .await?;
        let created: bool = row.try_get("created")?;

        let json_object: serde_json::Value = row.try_get("object")?;
        let object: FuncBinding = if created {
            let _history_event = HistoryEvent::new(
                ctx,
                FuncBinding::history_event_label(vec!["create"]),
                FuncBinding::history_event_message("created"),
                &serde_json::json![{ "visibility": ctx.visibility() }],
            )
            .await?;
            let object: FuncBinding = serde_json::from_value(json_object)?;
            object.set_func(ctx, &func_id).await?;
            object
        } else {
            serde_json::from_value(json_object)?
        };

        Ok((object, created))
    }

    standard_model_accessor!(args, PlainJson<JsonValue>, FuncBindingResult);
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
        ctx: &DalContext<'_, '_>,
    ) -> FuncBindingResult<FuncBindingReturnValue> {
        // NOTE: This is probably a bug in how we relate to function execution. This
        // fixes an issue where we need to have all the function return values, at the very
        // least, be in in the universal tenancy in order to look up values that are
        // identical. To be fixed later.
        let mut octx = ctx.clone();
        let write_tenancy = octx.write_tenancy().clone().into_universal();
        octx.update_write_tenancy(write_tenancy);
        let ctx = &octx;

        let func = self
            .func_with_tenancy(ctx)
            .await?
            .ok_or(FuncBindingError::FuncNotFound(self.pk))?;

        let mut execution = FuncExecution::new(ctx, &func, self).await?;

        let return_value = match self.backend_kind() {
            FuncBackendKind::Array => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendArrayArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendArray::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::Boolean => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendBooleanArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendBoolean::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::Identity => {
                let args: FuncBackendIdentityArgs = serde_json::from_value(self.args.clone())?;
                Some(args.identity)
            }
            FuncBackendKind::Integer => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendIntegerArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendInteger::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::JsCodeGeneration => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Dispatch)
                    .await?;

                let (tx, rx) = mpsc::channel(64);

                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;

                let handler = func
                    .handler()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let code_base64 = func
                    .code_base64()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;

                let args: FuncBackendJsCodeGenerationArgs =
                    serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendJsCodeGeneration::new(
                    ctx.veritech().clone(),
                    tx,
                    handler.to_owned(),
                    args,
                    code_base64.to_owned(),
                )
                .execute()
                .await?;

                let veritech_result = CodeGenerationResultSuccess::deserialize(&return_value)?;
                execution.process_output(ctx, rx).await?;
                Some(serde_json::to_value(&veritech_result.data)?)
            }
            FuncBackendKind::JsQualification => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Dispatch)
                    .await?;

                let (tx, rx) = mpsc::channel(64);

                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;

                let handler = func
                    .handler()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let code_base64 = func
                    .code_base64()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;

                let mut args: FuncBackendJsQualificationArgs =
                    serde_json::from_value(self.args.clone())?;
                ComponentView::reencrypt_secrets(ctx, &mut args.component.data).await?;
                for parent in &mut args.component.parents {
                    ComponentView::reencrypt_secrets(ctx, parent).await?;
                }

                let return_value = FuncBackendJsQualification::new(
                    ctx.veritech().clone(),
                    tx.clone(),
                    handler.to_owned(),
                    args,
                    code_base64.to_owned(),
                )
                .execute()
                .await?;

                let veritech_result = QualificationCheckResultSuccess::deserialize(&return_value)?;
                if let Some(message) = veritech_result.message {
                    tx.send(OutputStream {
                        execution_id: veritech_result.execution_id,
                        stream: "return".to_owned(),
                        level: "info".to_owned(),
                        group: None,
                        data: None,
                        message,
                        timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                    })
                    .await?;
                }
                let qual_result = QualificationResult {
                    success: veritech_result.qualified,
                    title: veritech_result.title,
                    link: veritech_result.link,
                    sub_checks: veritech_result.sub_checks,
                };

                std::mem::drop(tx);
                execution.process_output(ctx, rx).await?;
                Some(serde_json::to_value(&qual_result)?)
            }
            FuncBackendKind::JsResourceSync => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Dispatch)
                    .await?;

                let (tx, rx) = mpsc::channel(64);

                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;

                let handler = func
                    .handler()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let code_base64 = func
                    .code_base64()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;

                let args: FuncBackendJsResourceSyncArgs =
                    serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendJsResourceSync::new(
                    ctx.veritech().clone(),
                    tx,
                    handler.to_owned(),
                    args,
                    code_base64.to_owned(),
                )
                .execute()
                .await?;

                let veritech_result = ResourceSyncResultSuccess::deserialize(&return_value)?;

                execution.process_output(ctx, rx).await?;
                Some(serde_json::to_value(&veritech_result)?)
            }
            FuncBackendKind::JsAttribute => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Dispatch)
                    .await?;

                let (tx, rx) = mpsc::channel(64);
                let handler = func
                    .handler()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;
                let code_base64 = func
                    .code_base64()
                    .ok_or(FuncBindingError::JsFuncNotFound(self.pk))?;

                let mut args: FuncBackendJsAttributeArgs =
                    serde_json::from_value(self.args.clone())?;
                ComponentView::reencrypt_secrets(ctx, &mut args.component.data).await?;
                for parent in &mut args.component.parents {
                    ComponentView::reencrypt_secrets(ctx, parent).await?;
                }

                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;

                let return_value = FuncBackendJsAttribute::new(
                    ctx.veritech().clone(),
                    tx,
                    handler.to_owned(),
                    args,
                    code_base64.to_owned(),
                )
                .execute()
                .await?;

                execution.process_output(ctx, rx).await?;
                Some(return_value)
            }
            FuncBackendKind::Map => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendMapArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendMap::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::PropObject => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendPropObjectArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendPropObject::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::String => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendStringArgs = serde_json::from_value(self.args.clone())?;
                let return_value = FuncBackendString::new(args).execute().await?;
                Some(return_value)
            }
            FuncBackendKind::Unset => None,
            FuncBackendKind::ValidateStringValue => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Run)
                    .await?;
                let args: FuncBackendValidateStringValueArgs =
                    serde_json::from_value(self.args.clone())?;
                Some(FuncBackendValidateStringValue::new(args).execute()?)
            }
        };

        let func_binding_return_value = FuncBindingReturnValue::upsert(
            ctx,
            return_value.clone(),
            return_value,
            *func.id(),
            self.id,
            execution.pk(),
        )
        .await?;

        execution
            .process_return_value(ctx, &func_binding_return_value)
            .await?;
        execution
            .set_state(ctx, super::execution::FuncExecutionState::Success)
            .await?;

        Ok(func_binding_return_value)
    }
}
