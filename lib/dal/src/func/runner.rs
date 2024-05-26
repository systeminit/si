use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use si_events::{
    ActionResultState, CasValue, ContentHash, FuncRun, FuncRunBuilder, FuncRunBuilderError,
    FuncRunId, FuncRunLog, FuncRunLogId, FuncRunValue,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech_client::{BeforeFunction, OutputStream, ResolverFunctionComponent};

use crate::{
    action::{
        prototype::{ActionPrototype, ActionPrototypeError},
        Action, ActionError,
    },
    attribute::value::AttributeValueError,
    func::backend::FuncBackendError,
    secret::{before_funcs_for_component, BeforeFuncError},
    ActionPrototypeId, AttributeValue, AttributeValueId, ChangeSet, ChangeSetError, Component,
    ComponentError, ComponentId, DalContext, Func, FuncBackendKind, FuncError, FuncId, WsEvent,
    WsEventError, WsEventResult, WsPayload,
};

use super::backend::{
    array::FuncBackendArray,
    boolean::FuncBackendBoolean,
    diff::FuncBackendDiff,
    identity::FuncBackendIdentity,
    integer::FuncBackendInteger,
    js_action::FuncBackendJsAction,
    js_attribute::{FuncBackendJsAttribute, FuncBackendJsAttributeArgs},
    js_reconciliation::FuncBackendJsReconciliation,
    js_schema_variant_definition::FuncBackendJsSchemaVariantDefinition,
    json::FuncBackendJson,
    map::FuncBackendMap,
    object::FuncBackendObject,
    string::FuncBackendString,
    validation::FuncBackendValidation,
    FuncBackend, FuncDispatch, FuncDispatchContext, InvalidResolverFunctionTypeError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncRunnerError {
    #[error("action error: {0}")]
    ActionError(#[from] Box<ActionError>),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("before funcs error: {0}")]
    BeforeFunc(#[from] BeforeFuncError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("direct authentication func execution is unsupported (must go through \"before funcs\"), found: {0}")]
    DirectAuthenticationFuncExecutionUnsupported(FuncId),
    #[error("direct validation funcs are no longer supported, found: {0}")]
    DirectValidationFuncsNoLongerSupported(FuncId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("function backend error: {0}")]
    FuncBackend(#[from] FuncBackendError),
    #[error("validation function intrinsic func is missing -- bug!")]
    FuncIntrinsicValidationMissing,
    #[error("func run builder error: {0}")]
    FuncRunBuilder(#[from] FuncRunBuilderError),
    #[error("invalid resolver function type: {0}")]
    InvalidResolverFunctionType(#[from] InvalidResolverFunctionTypeError),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("function run result failure: kind={kind}, message={message}, backend={backend}")]
    ResultFailure {
        kind: String,
        message: String,
        backend: String,
    },
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type FuncRunnerResult<T> = Result<T, FuncRunnerError>;

pub type FuncRunnerValueChannel = tokio::sync::oneshot::Receiver<FuncRunnerResult<FuncRunValue>>;

pub struct FuncRunner;

impl FuncRunner {
    pub async fn run_test(
        ctx: &DalContext,
        func: Func,
        args: serde_json::Value,
        component_id: ComponentId,
    ) -> FuncRunnerResult<(FuncRunId, FuncRunnerValueChannel)> {
        let function_args: CasValue = args.clone().into();
        let (function_args_cas_address, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;
        let before = before_funcs_for_component(ctx, component_id).await?;

        let func_run_create_time = Utc::now();
        let func_run_inner = FuncRunBuilder::default()
            .actor(ctx.events_actor())
            .tenancy(ctx.events_tenancy())
            .backend_kind(func.backend_kind.into())
            .backend_response_type(func.backend_response_type.into())
            .function_name(func.name.clone())
            .function_kind(func.kind.into())
            .function_display_name(func.display_name.clone())
            .function_description(func.description.clone())
            .function_link(func.link.clone())
            .function_args_cas_address(function_args_cas_address)
            .function_code_cas_address(func.code_blake3)
            .attribute_value_id(None)
            .component_id(Some(component_id.into()))
            .created_at(func_run_create_time)
            .updated_at(func_run_create_time)
            .build()?;

        let func_run = Arc::new(func_run_inner);

        ctx.layer_db()
            .func_run()
            .write(
                func_run.clone(),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let func_run_id = func_run.id();
        let result_channel = FuncRunner::execute(ctx.clone(), func_run, func, args, before).await;

        Ok((func_run_id, result_channel))
    }

    pub async fn run_asset_definition_func(
        ctx: &DalContext,
        func: &Func,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        let args = serde_json::Value::Null;

        let function_args: CasValue = args.clone().into();

        let (function_args_cas_address, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
            let code_json_value: serde_json::Value = code.clone().into();
            let code_cas_value: CasValue = code_json_value.into();
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            hash
        } else {
            // Why are we doing this? Because the struct gods demand it. I have feelings.
            ContentHash::new("".as_bytes())
        };

        let func_run_create_time = Utc::now();
        let func_run_inner = FuncRunBuilder::default()
            .actor(ctx.events_actor())
            .tenancy(ctx.events_tenancy())
            .backend_kind(func.backend_kind.into())
            .backend_response_type(func.backend_response_type.into())
            .function_name(func.name.clone())
            .function_kind(func.kind.into())
            .function_display_name(func.display_name.clone())
            .function_description(func.description.clone())
            .function_link(func.link.clone())
            .function_args_cas_address(function_args_cas_address)
            .function_code_cas_address(code_cas_hash)
            .attribute_value_id(None)
            .component_id(None)
            .created_at(func_run_create_time)
            .updated_at(func_run_create_time)
            .build()?;

        let func_run = Arc::new(func_run_inner);

        ctx.layer_db()
            .func_run()
            .write(
                func_run.clone(),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let result_channel =
            FuncRunner::execute(ctx.clone(), func_run, func.clone(), args, vec![]).await;

        Ok(result_channel)
    }

    pub async fn run_validation_format(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        validation_format: String,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        let func_id =
            Func::find_intrinsic(ctx, super::intrinsics::IntrinsicFunc::Validation).await?;
        let func = Func::get_by_id(ctx, func_id)
            .await?
            .ok_or(FuncRunnerError::FuncIntrinsicValidationMissing)?;

        let args = serde_json::json!({
            "value": value,
            "validation_format": validation_format,
        });

        let function_args: CasValue = args.clone().into();

        let (function_args_cas_address, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
            let code_json_value: serde_json::Value = code.clone().into();
            let code_cas_value: CasValue = code_json_value.into();
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            hash
        } else {
            // Why are we doing this? Because the struct gods demand it. I have feelings.
            ContentHash::new("".as_bytes())
        };

        let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;

        let func_run_create_time = Utc::now();
        let func_run_inner = FuncRunBuilder::default()
            .actor(ctx.events_actor())
            .tenancy(ctx.events_tenancy())
            .backend_kind(func.backend_kind.into())
            .backend_response_type(func.backend_response_type.into())
            .function_name(func.name.clone())
            .function_kind(func.kind.into())
            .function_display_name(func.display_name.clone())
            .function_description(func.description.clone())
            .function_link(func.link.clone())
            .function_args_cas_address(function_args_cas_address)
            .function_code_cas_address(code_cas_hash)
            .attribute_value_id(Some(attribute_value_id.into()))
            .component_id(Some(component_id.into()))
            .created_at(func_run_create_time)
            .updated_at(func_run_create_time)
            .build()?;

        let func_run = Arc::new(func_run_inner);

        ctx.layer_db()
            .func_run()
            .write(
                func_run.clone(),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let result_channel = FuncRunner::execute(ctx.clone(), func_run, func, args, vec![]).await;

        Ok(result_channel)
    }

    pub async fn run_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        func_id: FuncId,
        args: serde_json::Value,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;

        let function_args: CasValue = args.clone().into();
        let (function_args_cas_address, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
            let code_json_value: serde_json::Value = code.clone().into();
            let code_cas_value: CasValue = code_json_value.into();
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            hash
        } else {
            // Why are we doing this? Because the struct gods demand it. I have feelings.
            ContentHash::new("".as_bytes())
        };

        let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
        let before = before_funcs_for_component(ctx, component_id).await?;

        let func_run_create_time = Utc::now();
        let func_run_inner = FuncRunBuilder::default()
            .actor(ctx.events_actor())
            .tenancy(ctx.events_tenancy())
            .backend_kind(func.backend_kind.into())
            .backend_response_type(func.backend_response_type.into())
            .function_name(func.name.clone())
            .function_kind(func.kind.into())
            .function_display_name(func.display_name.clone())
            .function_description(func.description.clone())
            .function_link(func.link.clone())
            .function_args_cas_address(function_args_cas_address)
            .function_code_cas_address(code_cas_hash)
            .attribute_value_id(Some(attribute_value_id.into()))
            .component_id(Some(component_id.into()))
            .created_at(func_run_create_time)
            .updated_at(func_run_create_time)
            .build()?;

        let func_run = Arc::new(func_run_inner);

        ctx.layer_db()
            .func_run()
            .write(
                func_run.clone(),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let result_channel = FuncRunner::execute(ctx.clone(), func_run, func, args, before).await;

        Ok(result_channel)
    }

    pub async fn run_action(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        component_id: ComponentId,
        func_id: FuncId,
        args: serde_json::Value,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        let prototype = ActionPrototype::get_by_id(ctx, action_prototype_id)
            .await
            .map_err(Box::new)?;
        let maybe_action_id = Action::find_equivalent(ctx, action_prototype_id, Some(component_id))
            .await
            .map_err(Box::new)?;
        let maybe_action_originating_change_set_id = match maybe_action_id {
            Some(action_id) => {
                let action = Action::get_by_id(ctx, action_id).await.map_err(Box::new)?;
                Some(action.originating_changeset_id())
            }
            None => None,
        };
        let maybe_action_originating_change_set_name = if let Some(
            action_originating_change_set_id,
        ) = maybe_action_originating_change_set_id
        {
            if let Some(original_change_set) =
                ChangeSet::find(ctx, action_originating_change_set_id).await?
            {
                Some(original_change_set.name)
            } else {
                None
            }
        } else {
            None
        };

        let function_args: CasValue = args.clone().into();
        let (function_args_cas_address, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
            let code_json_value: serde_json::Value = code.clone().into();
            let code_cas_value: CasValue = code_json_value.into();
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            hash
        } else {
            // Why are we doing this? Because the struct gods demand it. I have feelings.
            ContentHash::new("".as_bytes())
        };

        let before = before_funcs_for_component(ctx, component_id).await?;
        let component = Component::get_by_id(ctx, component_id).await?;
        let component_name = component.name(ctx).await?;
        let schema_name = component.schema(ctx).await?.name;

        let func_run_create_time = Utc::now();
        let func_run_inner = FuncRunBuilder::default()
            .actor(ctx.events_actor())
            .tenancy(ctx.events_tenancy())
            .backend_kind(func.backend_kind.into())
            .backend_response_type(func.backend_response_type.into())
            .function_name(func.name.clone())
            .function_kind(func.kind.into())
            .function_display_name(func.display_name.clone())
            .function_description(func.description.clone())
            .function_link(func.link.clone())
            .function_args_cas_address(function_args_cas_address)
            .function_code_cas_address(code_cas_hash)
            .action_id(maybe_action_id.map(|a| a.into()))
            .action_prototype_id(Some(action_prototype_id.into()))
            .action_kind(Some(prototype.kind.into()))
            .action_display_name(Some(prototype.name().clone()))
            .action_originating_change_set_id(
                maybe_action_originating_change_set_id.map(|a| a.into()),
            )
            .action_originating_change_set_name(maybe_action_originating_change_set_name)
            .action_result_state(Some(ActionResultState::Unknown))
            .attribute_value_id(None)
            .component_id(Some(component_id.into()))
            .component_name(Some(component_name))
            .schema_name(Some(schema_name))
            .created_at(func_run_create_time)
            .updated_at(func_run_create_time)
            .build()?;

        let func_run = Arc::new(func_run_inner);

        ctx.layer_db()
            .func_run()
            .write(
                func_run.clone(),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let result_channel = FuncRunner::execute(ctx.clone(), func_run, func, args, before).await;

        Ok(result_channel)
    }

    async fn execute(
        ctx: DalContext,
        func_run: Arc<FuncRun>,
        func: Func,
        args: serde_json::Value,
        before: Vec<BeforeFunction>,
    ) -> FuncRunnerValueChannel {
        let (tx, rx) = tokio::sync::oneshot::channel();

        // This probably needs a tracker, if we're being honest - but one thing at a time.
        tokio::spawn(async move {
            if let Err(error) = run_function_on_backend(tx, ctx, func_run, func, args, before).await
            {
                error!(?error, "Function Runner had an error during dispatch!");
            }

            async fn run_function_on_backend(
                tx: tokio::sync::oneshot::Sender<FuncRunnerResult<FuncRunValue>>,
                ctx: DalContext,
                func_run: Arc<FuncRun>,
                func: Func,
                args: serde_json::Value,
                before: Vec<BeforeFunction>,
            ) -> FuncRunnerResult<()> {
                let (func_dispatch_context, output_stream_rx) = FuncDispatchContext::new(&ctx);
                let func_run_id = func_run.id();

                // NOTE(nick): multi-track cloning! Bad idea? We'll see!
                let logs_ctx = ctx.clone();

                tokio::spawn(async move {
                    if let Err(error) = record_logs_from_running_function_on_backend(
                        logs_ctx,
                        func_run_id,
                        output_stream_rx,
                    )
                    .await
                    {
                        error!(
                            ?error,
                            "Function Runner had an error while recording logs from output stream!"
                        );
                    }

                    async fn record_logs_from_running_function_on_backend(
                        ctx: DalContext,
                        func_run_id: FuncRunId,
                        mut output_stream_rx: mpsc::Receiver<OutputStream>,
                    ) -> FuncRunnerResult<()> {
                        let mut func_run_log = FuncRunLog::new(func_run_id, ctx.events_tenancy());
                        while let Some(item) = output_stream_rx.recv().await {
                            func_run_log.push_log(si_events::OutputLine {
                                stream: item.stream,
                                execution_id: item.execution_id,
                                level: item.level,
                                group: item.group,
                                message: item.message,
                                timestamp: item.timestamp,
                            });

                            WsEvent::func_run_log_updated(
                                &ctx,
                                func_run_log.func_run_id(),
                                func_run_log.id(),
                            )
                            .await?
                            .publish_immediately(&ctx)
                            .await?;

                            ctx.layer_db()
                                .func_run_log()
                                .write(
                                    Arc::new(func_run_log.clone()),
                                    None,
                                    ctx.events_tenancy(),
                                    ctx.events_actor(),
                                )
                                .await?;
                        }

                        // Now that all `OutputStream` messages have been received, we will never
                        // see any further lines so we know we've reached the end. Marking the
                        // `FuncRunLog` as finalized can signal to other observers that this value
                        // is now effectively immutable and will not change going forward. (hint:
                        // this could be used to synchronize/wait on the final accumulation of all
                        // logs from a function execution, even if this task takes far longer than
                        // the execution time of the function).
                        func_run_log.set_finalized();
                        ctx.layer_db()
                            .func_run_log()
                            .write(
                                Arc::new(func_run_log.clone()),
                                None,
                                ctx.events_tenancy(),
                                ctx.events_actor(),
                            )
                            .await?;

                        Ok(())
                    }
                });

                let mut running_state_func_run_inner = Arc::unwrap_or_clone(func_run.clone());
                running_state_func_run_inner.set_state_to_running();
                let running_state_func_run = Arc::new(running_state_func_run_inner);
                ctx.layer_db()
                    .func_run()
                    .write(
                        running_state_func_run.clone(),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )
                    .await?;

                let execution_result = match func_run.backend_kind().into() {
                    FuncBackendKind::JsAction => {
                        FuncBackendJsAction::create_and_execute(
                            func_dispatch_context,
                            &func,
                            &args,
                            before,
                        )
                        .await
                    }
                    FuncBackendKind::JsReconciliation => {
                        FuncBackendJsReconciliation::create_and_execute(
                            func_dispatch_context,
                            &func,
                            &args,
                            before,
                        )
                        .await
                    }
                    FuncBackendKind::JsAttribute => {
                        let args = FuncBackendJsAttributeArgs {
                            component: ResolverFunctionComponent {
                                data: veritech_client::ComponentView {
                                    properties: args.to_owned(),
                                    ..Default::default()
                                },
                                parents: Vec::new(),
                            },
                            response_type: func.backend_response_type.try_into()?,
                        };
                        FuncBackendJsAttribute::create_and_execute(
                            func_dispatch_context,
                            &func,
                            &serde_json::to_value(args)?,
                            before,
                        )
                        .await
                    }
                    FuncBackendKind::JsSchemaVariantDefinition => {
                        FuncBackendJsSchemaVariantDefinition::create_and_execute(
                            func_dispatch_context,
                            &func,
                            &serde_json::Value::Null,
                            before,
                        )
                        .await
                    }
                    FuncBackendKind::Json => FuncBackendJson::create_and_execute(&args).await,
                    FuncBackendKind::Array => FuncBackendArray::create_and_execute(&args).await,
                    FuncBackendKind::Boolean => FuncBackendBoolean::create_and_execute(&args).await,
                    FuncBackendKind::Identity => {
                        FuncBackendIdentity::create_and_execute(&args).await
                    }
                    FuncBackendKind::Diff => FuncBackendDiff::create_and_execute(&args).await,
                    FuncBackendKind::Integer => FuncBackendInteger::create_and_execute(&args).await,
                    FuncBackendKind::Map => FuncBackendMap::create_and_execute(&args).await,
                    FuncBackendKind::Object => FuncBackendObject::create_and_execute(&args).await,
                    FuncBackendKind::String => FuncBackendString::create_and_execute(&args).await,
                    FuncBackendKind::Unset => Ok((None, None)),
                    FuncBackendKind::Validation => {
                        FuncBackendValidation::create_and_execute(
                            func_dispatch_context,
                            &func,
                            &args,
                            before,
                        )
                        .await
                    }
                    FuncBackendKind::JsValidation => {
                        return Err(FuncRunnerError::DirectValidationFuncsNoLongerSupported(
                            func.id,
                        ))
                    }
                    FuncBackendKind::JsAuthentication => {
                        return Err(
                            FuncRunnerError::DirectAuthenticationFuncExecutionUnsupported(func.id),
                        )
                    }
                };

                // if func.name.as_str() != "si:setObject" {
                //     dbg!(&func.name, &func.kind, &args, &execution_result);
                // }

                match execution_result {
                    Ok((mut unprocessed_value, mut value)) => {
                        // We so sorry - this is the way that the old code
                        // worked. Basically, we were serializing
                        // serde_json::Value::Null into the database when we
                        // executed functions, and then when you read it
                        // back out, it was as an Option<serde_json::Value>,
                        // which Serde would helpfuly translate into
                        // None, rather than Some(serde_json::Value::Null).
                        //
                        // Unfortunately, that means that there is a whole bunch
                        // of code that relies on basically never being able
                        // to read serde_json::Value::Null. So that's what
                        // we are going to do.
                        //
                        // Love,
                        // Adam, Fletcher, and Nick. :)

                        if unprocessed_value == Some(serde_json::Value::Null) {
                            unprocessed_value = None;
                        }

                        if value == Some(serde_json::Value::Null) {
                            value = None;
                        }

                        let mut next_state_inner =
                            Arc::unwrap_or_clone(running_state_func_run.clone());
                        next_state_inner.set_state_to_post_processing();
                        let next_state = Arc::new(next_state_inner);
                        ctx.layer_db()
                            .func_run()
                            .write(
                                next_state.clone(),
                                None,
                                ctx.events_tenancy(),
                                ctx.events_actor(),
                            )
                            .await?;
                        let _ = tx.send(Ok(FuncRunValue::new(
                            next_state.id(),
                            unprocessed_value,
                            value,
                        )));
                    }
                    Err(FuncBackendError::ResultFailure {
                        kind,
                        message,
                        backend,
                    }) => {
                        let mut next_state_inner =
                            Arc::unwrap_or_clone(running_state_func_run.clone());
                        next_state_inner.set_state_to_failure();
                        let next_state = Arc::new(next_state_inner);
                        ctx.layer_db()
                            .func_run()
                            .write(
                                next_state.clone(),
                                None,
                                ctx.events_tenancy(),
                                ctx.events_actor(),
                            )
                            .await?;

                        let _ = tx.send(Err(FuncRunnerError::ResultFailure {
                            kind,
                            message,
                            backend,
                        }));
                    }
                    Err(err) => {
                        let mut next_state_inner =
                            Arc::unwrap_or_clone(running_state_func_run.clone());
                        next_state_inner.set_state_to_failure();
                        let next_state = Arc::new(next_state_inner);
                        ctx.layer_db()
                            .func_run()
                            .write(
                                next_state.clone(),
                                None,
                                ctx.events_tenancy(),
                                ctx.events_actor(),
                            )
                            .await?;
                        let _ = tx.send(Err(err.into()));
                    }
                }
                Ok(())
            }
        });

        rx
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncRunLogUpdatedPayload {
    func_run_id: FuncRunId,
    func_run_log_id: FuncRunLogId,
}

impl WsEvent {
    pub async fn func_run_log_updated(
        ctx: &DalContext,
        func_run_id: FuncRunId,
        func_run_log_id: FuncRunLogId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncRunLogUpdated(FuncRunLogUpdatedPayload {
                func_run_id,
                func_run_log_id,
            }),
        )
        .await
    }
}
