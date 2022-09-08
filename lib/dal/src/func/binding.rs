use crate::{FuncError, WriteTenancy};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech::{OutputStream, ResolverFunctionComponent};

use crate::func::backend::{
    array::FuncBackendArray,
    boolean::FuncBackendBoolean,
    identity::FuncBackendIdentity,
    integer::FuncBackendInteger,
    js_attribute::{FuncBackendJsAttribute, FuncBackendJsAttributeArgs},
    js_code_generation::FuncBackendJsCodeGeneration,
    js_command::FuncBackendJsCommand,
    js_qualification::FuncBackendJsQualification,
    js_resource::FuncBackendJsResourceSync,
    js_workflow::FuncBackendJsWorkflow,
    map::FuncBackendMap,
    prop_object::FuncBackendPropObject,
    string::FuncBackendString,
    validation::FuncBackendValidateStringValue,
    FuncBackend, FuncDispatch, FuncDispatchContext,
};
use crate::DalContext;
use crate::{
    component::ComponentViewError, impl_standard_model, pk, standard_model,
    standard_model_accessor, standard_model_belongs_to, Func, FuncBackendError, FuncBackendKind,
    HistoryEvent, HistoryEventError, ReadTenancyError, StandardModel, StandardModelError,
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
    #[error("func error: {0}")]
    Func(#[from] FuncError),
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
        ctx: &DalContext<'_, '_, '_>,
        args: serde_json::Value,
        func_id: FuncId,
        backend_kind: FuncBackendKind,
    ) -> FuncBindingResult<Self> {
        let mut octx = ctx.clone();
        let write_tenancy = octx.write_tenancy().clone().into_universal();
        octx.update_write_tenancy(write_tenancy);
        let ctx = &octx;

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
        ctx: &DalContext<'_, '_, '_>,
        args: serde_json::Value,
        func_id: FuncId,
        backend_kind: FuncBackendKind,
    ) -> FuncBindingResult<(Self, bool)> {
        let mut octx = ctx.clone();
        let write_tenancy = octx.write_tenancy().clone().into_universal();
        octx.update_write_tenancy(write_tenancy);
        let ctx = &octx;

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object, created FROM func_binding_find_or_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.read_tenancy(),
                    ctx.write_tenancy(),
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

    /// Runs [`Self::find_or_create()`] and executes if [`Self`] was newly created or
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) could not be found.
    ///
    /// Use this function if you would like to receive the
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) for a given
    /// [`FuncId`](crate::Func) and [`args`](serde_json::Value).
    pub async fn find_or_create_and_execute(
        ctx: &DalContext<'_, '_, '_>,
        args: serde_json::Value,
        func_id: FuncId,
    ) -> FuncBindingResult<(Self, FuncBindingReturnValue)> {
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        let (func_binding, created) =
            Self::find_or_create(ctx, args, func_id, func.backend_kind).await?;

        let func_binding_return_value: FuncBindingReturnValue = if created {
            func_binding.execute(ctx).await?
        } else {
            // If the func binding was not newly created, let's try to find a func binding
            // return value first.
            let maybe_func_binding_return_value =
                FuncBindingReturnValue::get_by_func_binding_id(ctx, *func_binding.id()).await?;
            if let Some(func_binding_return_value) = maybe_func_binding_return_value {
                // If we found one, return it!
                func_binding_return_value
            } else {
                // If we did not find one, let's execute.
                func_binding.execute(ctx).await?
            }
        };

        Ok((func_binding, func_binding_return_value))
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

    // For a given [`FuncBinding`](Self), execute using veritech.
    pub async fn execute(
        &self,
        ctx: &DalContext<'_, '_, '_>,
    ) -> FuncBindingResult<FuncBindingReturnValue> {
        let (func, execution, context, mut rx) = self.prepare_execution(ctx).await?;
        let value = self.execute_critical_section(func.clone(), context).await?;

        let mut output = Vec::new();
        while let Some(output_stream) = rx.recv().await {
            output.push(output_stream);
        }

        self.postprocess_execution(ctx, output, &func, value, execution)
            .await
    }

    /// Perform function execution to veritech for a given [`Func`](crate::Func) and
    /// [`FuncDispatchContext`](crate::func::backend::FuncDispatchContext).
    pub async fn execute_critical_section(
        &self,
        func: Func,
        context: FuncDispatchContext,
    ) -> FuncBindingResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        // TODO: encrypt components
        let value = match self.backend_kind() {
            FuncBackendKind::JsCodeGeneration => {
                FuncBackendJsCodeGeneration::create_and_execute(context, &func, &self.args).await?
            }
            FuncBackendKind::JsQualification => {
                FuncBackendJsQualification::create_and_execute(context, &func, &self.args).await?
            }
            FuncBackendKind::JsWorkflow => {
                FuncBackendJsWorkflow::create_and_execute(context, &func, &self.args).await?
            }
            FuncBackendKind::JsCommand => {
                FuncBackendJsCommand::create_and_execute(context, &func, &self.args).await?
            }
            FuncBackendKind::JsResourceSync => {
                FuncBackendJsResourceSync::create_and_execute(context, &func, &self.args).await?
            }
            FuncBackendKind::JsAttribute => {
                FuncBackendJsAttribute::create_and_execute(context, &func, &self.args).await?
            }
            FuncBackendKind::Json => {
                let component = FuncBackendJsAttributeArgs {
                    component: ResolverFunctionComponent {
                        data: veritech::ComponentView {
                            properties: self.args.clone(),
                            ..Default::default()
                        },
                        parents: Vec::new(),
                    },
                };
                FuncBackendJsAttribute::create_and_execute(
                    context,
                    &func,
                    &serde_json::to_value(component)?,
                )
                .await?
            }
            FuncBackendKind::Array => FuncBackendArray::create_and_execute(&self.args).await?,
            FuncBackendKind::Boolean => FuncBackendBoolean::create_and_execute(&self.args).await?,
            FuncBackendKind::Identity => {
                FuncBackendIdentity::create_and_execute(&self.args).await?
            }
            FuncBackendKind::Integer => FuncBackendInteger::create_and_execute(&self.args).await?,
            FuncBackendKind::Map => FuncBackendMap::create_and_execute(&self.args).await?,
            FuncBackendKind::PropObject => {
                FuncBackendPropObject::create_and_execute(&self.args).await?
            }
            FuncBackendKind::String => FuncBackendString::create_and_execute(&self.args).await?,
            FuncBackendKind::Unset => (None, None),
            FuncBackendKind::ValidateStringValue => {
                FuncBackendValidateStringValue::create_and_execute(&self.args).await?
            }
        };
        Ok(value)
    }

    pub async fn postprocess_execution(
        &self,
        ctx: &DalContext<'_, '_, '_>,
        output_stream: Vec<OutputStream>,
        func: &Func,
        (unprocessed_value, processed_value): (
            Option<serde_json::Value>,
            Option<serde_json::Value>,
        ),
        mut execution: FuncExecution,
    ) -> FuncBindingResult<FuncBindingReturnValue> {
        execution.set_output_stream(ctx, output_stream).await?;

        let func_binding_return_value = FuncBindingReturnValue::upsert(
            ctx,
            unprocessed_value,
            processed_value,
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

    pub async fn prepare_execution(
        &self,
        ctx: &DalContext<'_, '_, '_>,
    ) -> FuncBindingResult<(
        Func,
        FuncExecution,
        FuncDispatchContext,
        mpsc::Receiver<OutputStream>,
    )> {
        // NOTE: This is probably a bug in how we relate to function execution. This
        // fixes an issue where we need to have all the function return values, at the very
        // least, be in in the universal tenancy in order to look up values that are
        // identical. To be fixed later.
        let mut octx = ctx.clone();
        let write_tenancy = octx.write_tenancy().clone().into_universal();
        octx.update_write_tenancy(write_tenancy);
        let ctx = &octx;

        let func: Func = self
            .func(ctx)
            .await?
            .ok_or(FuncBindingError::FuncNotFound(self.pk))?;

        let mut execution = FuncExecution::new(ctx, &func, self).await?;

        match self.backend_kind() {
            FuncBackendKind::Array
            | FuncBackendKind::Boolean
            | FuncBackendKind::Identity
            | FuncBackendKind::Integer
            | FuncBackendKind::Map
            | FuncBackendKind::PropObject
            | FuncBackendKind::String
            | FuncBackendKind::Unset
            | FuncBackendKind::ValidateStringValue => {}

            FuncBackendKind::JsResourceSync
            | FuncBackendKind::JsCodeGeneration
            | FuncBackendKind::JsQualification
            | FuncBackendKind::JsAttribute
            | FuncBackendKind::JsWorkflow
            | FuncBackendKind::JsCommand
            | FuncBackendKind::Json => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Dispatch)
                    .await?;
            }
        }

        // NOTE(nick,wendy): why is the state is set to Run immediately after it is set to
        // Dispatch a few lines above?
        execution
            .set_state(ctx, super::execution::FuncExecutionState::Run)
            .await?;

        let (context, rx) = FuncDispatchContext::new(ctx);
        Ok((func, execution, context, rx))
    }
}
