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
use tokio::sync::{mpsc, oneshot};
use veritech_client::{
    encrypt_value_tree, BeforeFunction, OutputStream, ResolverFunctionComponent,
    VeritechValueEncryptError,
};

use crate::prop::{PropError, PropPath};
use crate::schema::variant::root_prop::RootPropChild;
use crate::{
    action::{
        prototype::{ActionPrototype, ActionPrototypeError},
        Action, ActionError,
    },
    attribute::value::AttributeValueError,
    func::backend::FuncBackendError,
    ActionPrototypeId, AttributeValue, AttributeValueId, ChangeSet, ChangeSetError, Component,
    ComponentError, ComponentId, DalContext, EncryptedSecret, Func, FuncBackendKind, FuncError,
    FuncId, Prop, PropId, SchemaVariant, SchemaVariantError, Secret, SecretError, WsEvent,
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
    #[error("before func missing expected code: {0}")]
    BeforeFuncMissingCode(FuncId),
    #[error("before func missing expected handler: {0}")]
    BeforeFuncMissingHandler(FuncId),
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
    #[error("no widget options for secret prop id: {0}")]
    NoWidgetOptionsForSecretProp(PropId),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("function run result failure: kind={kind}, message={message}, backend={backend}")]
    ResultFailure {
        kind: String,
        message: String,
        backend: String,
    },
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("veritech value encrypt error: {0}")]
    VeritechValueEncrypt(#[from] VeritechValueEncryptError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type FuncRunnerResult<T> = Result<T, FuncRunnerError>;

pub type FuncRunnerValueChannel = tokio::sync::oneshot::Receiver<FuncRunnerResult<FuncRunValue>>;

pub struct FuncRunner {
    func_run: Arc<FuncRun>,
    func: Func,
    args: serde_json::Value,
    before: Vec<BeforeFunction>,
}

impl FuncRunner {
    #[instrument(
        name = "func_runner.run_test",
        level = "debug",
        skip_all,
        fields(
            job.id = Empty,
            job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = func.name.as_str(),
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.change_set.id = Empty,
            si.component.id = Empty,
            si.func_run.func.args = Empty,
            si.func_run.func.backend_kind = func.backend_kind.as_ref(),
            si.func_run.func.backend_response_type = func.backend_response_type.as_ref(),
            si.func_run.func.id = Empty,
            si.func_run.func.kind = func.kind.as_ref(),
            si.func_run.func.name = func.name.as_str(),
            si.func_run.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn run_test(
        ctx: &DalContext,
        func: Func,
        args: serde_json::Value,
        component_id: ComponentId,
    ) -> FuncRunnerResult<(FuncRunId, FuncRunnerValueChannel)> {
        // Prepares the function for execution.
        //
        // Note: this function is internal so we can record early-returning errors in span metadata
        // and in order to time the function's preparation vs. execution timings.
        #[instrument(
            name = "func_runner.run_test.prepare",
            level = "debug",
            skip_all,
            fields()
        )]
        #[inline]
        async fn prepare(
            ctx: &DalContext,
            func: Func,
            args: serde_json::Value,
            component_id: ComponentId,
            span: &Span,
        ) -> FuncRunnerResult<FuncRunner> {
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
            let before = FuncRunner::before_funcs(ctx, component_id).await?;

            let component_id = component_id.into();

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
                .component_id(Some(component_id))
                .created_at(func_run_create_time)
                .updated_at(func_run_create_time)
                .build()?;

            if !span.is_disabled() {
                let mut id_buf = FuncRunId::array_to_str_buf();

                let id = func_run_inner.id().array_to_str(&mut id_buf);
                span.record("job.id", &id);
                span.record("si.func_run.id", &id);

                let invoked_args = serde_json::to_string(&args)
                    .unwrap_or_else(|_| "args failed to serialize".to_owned());
                span.record("job.invoked_args", invoked_args.as_str());
                span.record("si.func_run.func.args", invoked_args.as_str());

                span.record("si.func_run.func.id", func.id.array_to_str(&mut id_buf));

                span.record(
                    "si.change_set.id",
                    func_run_inner.change_set_id().array_to_str(&mut id_buf),
                );
                span.record("si.component.id", component_id.array_to_str(&mut id_buf));
                span.record(
                    "si.workspace.id",
                    func_run_inner.workspace_pk().array_to_str(&mut id_buf),
                );
            }

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

            Ok(FuncRunner {
                func_run,
                func,
                args,
                before,
            })
        }

        let span = Span::current();

        let runner = prepare(ctx, func, args, component_id, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let func_run_id = runner.id();
        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok((func_run_id, result_channel))
    }

    #[instrument(
        name = "func_runner.run_asset_definition_func",
        level = "debug",
        skip_all,
        fields(
            job.id = Empty,
            job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = func.name.as_str(),
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.action.id = Empty,
            si.action.kind = Empty,
            si.change_set.id = Empty,
            si.func_run.func.args = Empty,
            si.func_run.func.backend_kind = func.backend_kind.as_ref(),
            si.func_run.func.backend_response_type = func.backend_response_type.as_ref(),
            si.func_run.func.id = Empty,
            si.func_run.func.kind = func.kind.as_ref(),
            si.func_run.func.name = func.name.as_str(),
            si.func_run.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn run_asset_definition_func(
        ctx: &DalContext,
        func: &Func,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        // Prepares the function for execution.
        //
        // Note: this function is internal so we can record early-returning errors in span metadata
        // and in order to time the function's preparation vs. execution timings.
        #[instrument(
            name = "func_runner.run_asset_definition_func.prepare",
            level = "debug",
            skip_all,
            fields()
        )]
        #[inline]
        async fn prepare(
            ctx: &DalContext,
            func: &Func,
            span: &Span,
        ) -> FuncRunnerResult<FuncRunner> {
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

            if !span.is_disabled() {
                let mut id_buf = FuncRunId::array_to_str_buf();

                let id = func_run_inner.id().array_to_str(&mut id_buf);
                span.record("job.id", &id);
                span.record("si.func_run.id", &id);

                let invoked_args = serde_json::to_string(&args)
                    .unwrap_or_else(|_| "args failed to serialize".to_owned());
                span.record("job.invoked_args", invoked_args.as_str());
                span.record("si.func_run.func.args", invoked_args.as_str());

                span.record("si.func_run.func.id", func.id.array_to_str(&mut id_buf));

                span.record(
                    "si.change_set.id",
                    func_run_inner.change_set_id().array_to_str(&mut id_buf),
                );
                span.record(
                    "si.workspace.id",
                    func_run_inner.workspace_pk().array_to_str(&mut id_buf),
                );
            }

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

            Ok(FuncRunner {
                func_run,
                func: func.clone(),
                args,
                before: vec![],
            })
        }

        let span = Span::current();

        let runner = prepare(ctx, func, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok(result_channel)
    }

    #[instrument(
        name = "func_runner.run_validation_format",
        level = "debug",
        skip_all,
        fields(
            job.id = Empty,
            job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = Empty,
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.attribute_value.id = Empty,
            si.change_set.id = Empty,
            si.component.id = Empty,
            si.func_run.func.args = Empty,
            si.func_run.func.backend_kind = Empty,
            si.func_run.func.backend_response_type = Empty,
            si.func_run.func.id = Empty,
            si.func_run.func.kind = Empty,
            si.func_run.func.name = Empty,
            si.func_run.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn run_validation_format(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        validation_format: String,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        // Prepares the function for execution.
        //
        // Note: this function is internal so we can record early-returning errors in span metadata
        // and in order to time the function's preparation vs. execution timings.
        #[instrument(
            name = "func_runner.run_validation_format.prepare",
            level = "debug",
            skip_all,
            fields()
        )]
        #[inline]
        async fn prepare(
            ctx: &DalContext,
            attribute_value_id: AttributeValueId,
            value: Option<serde_json::Value>,
            validation_format: String,
            span: &Span,
        ) -> FuncRunnerResult<FuncRunner> {
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
            let component_id = component_id.into();
            let attribute_value_id = attribute_value_id.into();

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
                .attribute_value_id(Some(attribute_value_id))
                .component_id(Some(component_id))
                .created_at(func_run_create_time)
                .updated_at(func_run_create_time)
                .build()?;

            if !span.is_disabled() {
                let mut id_buf = FuncRunId::array_to_str_buf();

                let id = func_run_inner.id().array_to_str(&mut id_buf);
                span.record("job.id", &id);
                span.record("si.func_run.id", &id);

                let invoked_args = serde_json::to_string(&args)
                    .unwrap_or_else(|_| "args failed to serialize".to_owned());
                span.record("job.invoked_args", invoked_args.as_str());
                span.record("si.func_run.func.args", invoked_args.as_str());

                span.record("job.invoked_name", func.name.as_str());
                span.record("si.func_run.func.name", func.name.as_str());

                span.record("si.func_run.func.backend_kind", func.backend_kind.as_ref());
                span.record(
                    "si.func_run.func.backend_response_type",
                    func.backend_response_type.as_ref(),
                );
                span.record("si.func_run.func.id", func.id.array_to_str(&mut id_buf));
                span.record("si.func_run.func.kind", func.kind.as_ref());

                span.record(
                    "si.attribute_value.id",
                    attribute_value_id.array_to_str(&mut id_buf),
                );
                span.record(
                    "si.change_set.id",
                    func_run_inner.change_set_id().array_to_str(&mut id_buf),
                );
                span.record("si.component.id", component_id.array_to_str(&mut id_buf));
                span.record(
                    "si.workspace.id",
                    func_run_inner.workspace_pk().array_to_str(&mut id_buf),
                );
            }

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

            Ok(FuncRunner {
                func_run,
                func,
                args,
                before: vec![],
            })
        }

        let span = Span::current();

        let runner = prepare(ctx, attribute_value_id, value, validation_format, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok(result_channel)
    }

    #[instrument(
        name = "func_runner.run_attribute_value",
        level = "debug",
        skip_all,
        fields(
            job.id = Empty,
            job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = Empty,
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.attribute_value.id = Empty,
            si.change_set.id = Empty,
            si.component.id = Empty,
            si.func_run.func.args = Empty,
            si.func_run.func.backend_kind = Empty,
            si.func_run.func.backend_response_type = Empty,
            si.func_run.func.id = Empty,
            si.func_run.func.kind = Empty,
            si.func_run.func.name = Empty,
            si.func_run.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn run_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        func_id: FuncId,
        args: serde_json::Value,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        // Prepares the function for execution.
        //
        // Note: this function is internal so we can record early-returning errors in span metadata
        // and in order to time the function's preparation vs. execution timings.
        #[instrument(
            name = "func_runner.run_attribute_value.prepare",
            level = "debug",
            skip_all,
            fields()
        )]
        #[inline]
        async fn prepare(
            ctx: &DalContext,
            attribute_value_id: AttributeValueId,
            func_id: FuncId,
            args: serde_json::Value,
            parent_span: &Span,
        ) -> FuncRunnerResult<FuncRunner> {
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
            let before = FuncRunner::before_funcs(ctx, component_id).await?;

            let component_id = component_id.into();
            let attribute_value_id = attribute_value_id.into();

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
                .attribute_value_id(Some(attribute_value_id))
                .component_id(Some(component_id))
                .created_at(func_run_create_time)
                .updated_at(func_run_create_time)
                .build()?;

            if !parent_span.is_disabled() {
                let mut id_buf = FuncRunId::array_to_str_buf();

                let id = func_run_inner.id().array_to_str(&mut id_buf);
                parent_span.record("job.id", &id);
                parent_span.record("si.func_run.id", &id);

                let invoked_args = serde_json::to_string(&args)
                    .unwrap_or_else(|_| "args failed to serialize".to_owned());
                parent_span.record("job.invoked_args", invoked_args.as_str());
                parent_span.record("si.func_run.func.args", invoked_args.as_str());

                parent_span.record("job.invoked_name", func.name.as_str());
                parent_span.record("si.func_run.func.name", func.name.as_str());

                parent_span.record("si.func_run.func.backend_kind", func.backend_kind.as_ref());
                parent_span.record(
                    "si.func_run.func.backend_response_type",
                    func.backend_response_type.as_ref(),
                );
                parent_span.record("si.func_run.func.id", func.id.array_to_str(&mut id_buf));
                parent_span.record("si.func_run.func.kind", func.kind.as_ref());

                parent_span.record(
                    "si.attribute_value.id",
                    attribute_value_id.array_to_str(&mut id_buf),
                );
                parent_span.record(
                    "si.change_set.id",
                    func_run_inner.change_set_id().array_to_str(&mut id_buf),
                );
                parent_span.record("si.component.id", component_id.array_to_str(&mut id_buf));
                parent_span.record(
                    "si.workspace.id",
                    func_run_inner.workspace_pk().array_to_str(&mut id_buf),
                );
            }

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

            Ok(FuncRunner {
                func_run,
                func,
                args,
                before,
            })
        }

        let span = Span::current();

        let runner = prepare(ctx, attribute_value_id, func_id, args, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok(result_channel)
    }

    #[instrument(
        name = "func_runner.run_action",
        level = "debug",
        skip_all,
        fields(
            job.id = Empty,
            job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = Empty,
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.action.id = Empty,
            si.action.kind = Empty,
            si.change_set.id = Empty,
            si.component.id = Empty,
            si.func_run.func.args = Empty,
            si.func_run.func.backend_kind = Empty,
            si.func_run.func.backend_response_type = Empty,
            si.func_run.func.id = Empty,
            si.func_run.func.kind = Empty,
            si.func_run.func.name = Empty,
            si.func_run.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn run_action(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        component_id: ComponentId,
        func_id: FuncId,
        args: serde_json::Value,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        // Prepares the function for execution.
        //
        // Note: this function is internal so we can record early-returning errors in span metadata
        // and in order to time the function's preparation vs. execution timings.
        #[instrument(
            name = "func_runner.run_action.prepare",
            level = "debug",
            skip_all,
            fields()
        )]
        #[inline]
        async fn prepare(
            ctx: &DalContext,
            action_prototype_id: ActionPrototypeId,
            component_id: ComponentId,
            func_id: FuncId,
            args: serde_json::Value,
            span: &Span,
        ) -> FuncRunnerResult<FuncRunner> {
            let func = Func::get_by_id_or_error(ctx, func_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, action_prototype_id)
                .await
                .map_err(Box::new)?;
            let maybe_action_id =
                Action::find_equivalent(ctx, action_prototype_id, Some(component_id))
                    .await
                    .map_err(Box::new)?;
            let maybe_action_originating_change_set_id = match maybe_action_id {
                Some(action_id) => {
                    let action = Action::get_by_id(ctx, action_id).await.map_err(Box::new)?;
                    Some(action.originating_changeset_id())
                }
                None => None,
            };
            let maybe_action_originating_change_set_name =
                if let Some(action_originating_change_set_id) =
                    maybe_action_originating_change_set_id
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

            let before = FuncRunner::before_funcs(ctx, component_id).await?;
            let component = Component::get_by_id(ctx, component_id).await?;
            let component_name = component.name(ctx).await?;
            let schema_name = component.schema(ctx).await?.name;

            let component_id = component_id.into();
            let action_kind = prototype.kind.into();

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
                .action_kind(Some(action_kind))
                .action_display_name(Some(prototype.name().clone()))
                .action_originating_change_set_id(
                    maybe_action_originating_change_set_id.map(|a| a.into()),
                )
                .action_originating_change_set_name(maybe_action_originating_change_set_name)
                .action_result_state(Some(ActionResultState::Unknown))
                .attribute_value_id(None)
                .component_id(Some(component_id))
                .component_name(Some(component_name))
                .schema_name(Some(schema_name))
                .created_at(func_run_create_time)
                .updated_at(func_run_create_time)
                .build()?;

            if !span.is_disabled() {
                let mut id_buf = FuncRunId::array_to_str_buf();

                let id = func_run_inner.id().array_to_str(&mut id_buf);
                span.record("job.id", &id);
                span.record("si.func_run.id", &id);

                let invoked_args = serde_json::to_string(&args)
                    .unwrap_or_else(|_| "args failed to serialize".to_owned());
                span.record("job.invoked_args", invoked_args.as_str());
                span.record("si.func_run.func.args", invoked_args.as_str());

                span.record("job.invoked_name", func.name.as_str());
                span.record("si.func_run.func.name", func.name.as_str());

                if let Some(action_id) = maybe_action_id {
                    span.record("si.action.id", action_id.array_to_str(&mut id_buf));
                }
                span.record("si.action.kind", action_kind.as_ref());
                span.record("si.func_run.func.backend_kind", func.backend_kind.as_ref());
                span.record(
                    "si.func_run.func.backend_response_type",
                    func.backend_response_type.as_ref(),
                );
                span.record("si.func_run.func.id", func.id.array_to_str(&mut id_buf));
                span.record("si.func_run.func.kind", func.kind.as_ref());

                span.record(
                    "si.change_set.id",
                    func_run_inner.change_set_id().array_to_str(&mut id_buf),
                );
                span.record("si.component.id", component_id.array_to_str(&mut id_buf));
                span.record(
                    "si.workspace.id",
                    func_run_inner.workspace_pk().array_to_str(&mut id_buf),
                );
            }

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

            Ok(FuncRunner {
                func_run,
                func,
                args,
                before,
            })
        }

        let span = Span::current();

        let runner = prepare(ctx, action_prototype_id, component_id, func_id, args, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok(result_channel)
    }

    fn id(&self) -> FuncRunId {
        self.func_run.id()
    }

    async fn execute(self, ctx: DalContext, execution_parent_span: Span) -> FuncRunnerValueChannel {
        let func_run_id = self.func_run.id();
        let (func_dispatch_context, output_stream_rx) = FuncDispatchContext::new(&ctx);
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        let logs_task = FuncRunnerLogsTask {
            ctx: ctx.clone(),
            func_run_id,
            output_stream_rx,
        };

        let execution_task = FuncRunnerExecutionTask {
            result_tx,
            ctx,
            func_dispatch_context,
            func_run: self.func_run,
            func: self.func,
            args: self.args,
            before: self.before,
            parent_span: execution_parent_span,
        };

        // This probably needs a tracker, if we're being honest - but one thing at a time.
        tokio::spawn(logs_task.run());
        tokio::spawn(execution_task.run());

        result_rx
    }

    /// This _private_ method collects all [`BeforeFunctions`](BeforeFunction) for a given
    /// [`ComponentId`](Component).
    #[instrument(name = "func_runner.before_funcs", level = "debug", skip_all)]
    async fn before_funcs(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> FuncRunnerResult<Vec<BeforeFunction>> {
        let secret_props = {
            let schema_variant = Component::schema_variant_id(ctx, component_id).await?;
            let secrets_prop =
                SchemaVariant::find_root_child_prop_id(ctx, schema_variant, RootPropChild::Secrets)
                    .await?;
            Prop::direct_child_prop_ids(ctx, secrets_prop).await?
        };

        let secret_definition_path = PropPath::new(["root", "secret_definition"]);
        let secret_path = PropPath::new(["root", "secrets"]);

        let mut funcs_and_secrets = vec![];
        for secret_prop_id in secret_props {
            let auth_funcs = Self::auth_funcs_for_secret_prop_id(
                ctx,
                secret_prop_id,
                &secret_definition_path,
                &secret_path,
            )
            .await?;

            let av_ids = Prop::attribute_values_for_prop_id(ctx, secret_prop_id).await?;
            let mut maybe_value = None;
            for av_id in av_ids {
                if AttributeValue::component_id(ctx, av_id).await? != component_id {
                    continue;
                }

                let av = AttributeValue::get_by_id(ctx, av_id).await?;

                maybe_value = av.value(ctx).await?;
                break;
            }

            if let Some(value) = maybe_value {
                let key = Secret::key_from_value_in_attribute_value(value)?;
                funcs_and_secrets.push((key, auth_funcs))
            }
        }

        let mut results = vec![];

        for (key, funcs) in funcs_and_secrets {
            let encrypted_secret = EncryptedSecret::get_by_key(ctx, key)
                .await?
                .ok_or(SecretError::EncryptedSecretNotFound(key))?;

            // Decrypt message from EncryptedSecret
            let mut arg = encrypted_secret.decrypt(ctx).await?.message().into_inner();

            // Re-encrypt raw Value for transmission to Veritech
            encrypt_value_tree(&mut arg, ctx.encryption_key())?;

            for func in funcs {
                results.push(BeforeFunction {
                    handler: func
                        .handler
                        .ok_or_else(|| FuncRunnerError::BeforeFuncMissingHandler(func.id))?,
                    code_base64: func
                        .code_base64
                        .ok_or_else(|| FuncRunnerError::BeforeFuncMissingCode(func.id))?,
                    arg: arg.clone(),
                })
            }
        }

        Ok(results)
    }

    /// This _private_ method gathers the authentication functions for a given [`PropId`](Prop)
    /// underneath "/root/secrets".
    #[instrument(
        name = "func_runner.before_funcs.auth_funcs_for_secret_prop_id",
        level = "debug",
        skip_all
    )]
    async fn auth_funcs_for_secret_prop_id(
        ctx: &DalContext,
        secret_prop_id: PropId,
        secret_definition_path: &PropPath,
        secret_path: &PropPath,
    ) -> FuncRunnerResult<Vec<Func>> {
        let secret_prop = Prop::get_by_id_or_error(ctx, secret_prop_id).await?;

        let secret_definition_name = secret_prop
            .widget_options
            .ok_or(FuncRunnerError::NoWidgetOptionsForSecretProp(
                secret_prop_id,
            ))?
            .pop()
            .ok_or(FuncRunnerError::NoWidgetOptionsForSecretProp(
                secret_prop_id,
            ))?
            .value;

        let mut auth_funcs = vec![];
        for secret_defining_sv_id in SchemaVariant::list_ids(ctx).await? {
            if Prop::find_prop_id_by_path_opt(ctx, secret_defining_sv_id, secret_definition_path)
                .await?
                .is_none()
            {
                continue;
            }

            let secrets_prop =
                Prop::find_prop_by_path(ctx, secret_defining_sv_id, secret_path).await?;

            let secret_child_prop_id =
                Prop::direct_single_child_prop_id(ctx, secrets_prop.id).await?;
            let secret_child_prop = Prop::get_by_id_or_error(ctx, secret_child_prop_id).await?;

            if secret_child_prop.name != secret_definition_name {
                continue;
            }

            for auth_func_id in
                SchemaVariant::list_auth_func_ids_for_id(ctx, secret_defining_sv_id).await?
            {
                auth_funcs.push(Func::get_by_id_or_error(ctx, auth_func_id).await?)
            }

            break;
        }

        Ok(auth_funcs)
    }
}

struct FuncRunnerLogsTask {
    ctx: DalContext,
    func_run_id: FuncRunId,
    output_stream_rx: mpsc::Receiver<OutputStream>,
}

impl FuncRunnerLogsTask {
    const NAME: &'static str = "Dal::FuncRunnerLogsTask";

    async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(task = Self::NAME, error = ?err, "error while processing function logs");
        }
    }

    async fn try_run(mut self) -> FuncRunnerResult<()> {
        let mut func_run_log = FuncRunLog::new(self.func_run_id, self.ctx.events_tenancy());
        while let Some(item) = self.output_stream_rx.recv().await {
            func_run_log.push_log(si_events::OutputLine {
                stream: item.stream,
                execution_id: item.execution_id,
                level: item.level,
                group: item.group,
                message: item.message,
                timestamp: item.timestamp,
            });

            WsEvent::func_run_log_updated(&self.ctx, func_run_log.func_run_id(), func_run_log.id())
                .await?
                .publish_immediately(&self.ctx)
                .await?;

            self.ctx
                .layer_db()
                .func_run_log()
                .write(
                    Arc::new(func_run_log.clone()),
                    None,
                    self.ctx.events_tenancy(),
                    self.ctx.events_actor(),
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
        self.ctx
            .layer_db()
            .func_run_log()
            .write(
                Arc::new(func_run_log.clone()),
                None,
                self.ctx.events_tenancy(),
                self.ctx.events_actor(),
            )
            .await?;

        Ok(())
    }
}

struct FuncRunnerExecutionTask {
    result_tx: oneshot::Sender<FuncRunnerResult<FuncRunValue>>,
    ctx: DalContext,
    func_dispatch_context: FuncDispatchContext,
    func_run: Arc<FuncRun>,
    func: Func,
    args: serde_json::Value,
    before: Vec<BeforeFunction>,
    parent_span: Span,
}

impl FuncRunnerExecutionTask {
    const NAME: &'static str = "Dal::FuncRunnerExecutionTask";

    #[instrument(
        name = "func_runner.execution_task.run",
        level = "debug",
        parent = &self.parent_span,
        skip_all,
        fields()
    )]
    async fn run(self) {
        let span = Span::current();
        let parent_span = self.parent_span.clone();

        if let Err(err) = self
            .try_run()
            .await
            .inspect(|_| {
                span.record_ok();
                parent_span.record_ok()
            })
            .map_err(|err| {
                let err = span.record_err(err);
                parent_span.record_err(err)
            })
        {
            error!(task = Self::NAME, error = ?err, "error while dispatching and running function");
        }
    }

    async fn try_run(self) -> FuncRunnerResult<()> {
        let mut running_state_func_run_inner = Arc::unwrap_or_clone(self.func_run.clone());
        running_state_func_run_inner.set_state_to_running();
        let running_state_func_run = Arc::new(running_state_func_run_inner);
        self.ctx
            .layer_db()
            .func_run()
            .write(
                running_state_func_run.clone(),
                None,
                self.ctx.events_tenancy(),
                self.ctx.events_actor(),
            )
            .await?;

        let execution_result = match self.func_run.backend_kind().into() {
            FuncBackendKind::JsAction => {
                FuncBackendJsAction::create_and_execute(
                    self.func_dispatch_context,
                    &self.func,
                    &self.args,
                    self.before,
                )
                .await
            }
            FuncBackendKind::JsReconciliation => {
                FuncBackendJsReconciliation::create_and_execute(
                    self.func_dispatch_context,
                    &self.func,
                    &self.args,
                    self.before,
                )
                .await
            }
            FuncBackendKind::JsAttribute => {
                let args = FuncBackendJsAttributeArgs {
                    component: ResolverFunctionComponent {
                        data: veritech_client::ComponentView {
                            properties: self.args.to_owned(),
                            ..Default::default()
                        },
                        parents: Vec::new(),
                    },
                    response_type: self.func.backend_response_type.try_into()?,
                };
                FuncBackendJsAttribute::create_and_execute(
                    self.func_dispatch_context,
                    &self.func,
                    &serde_json::to_value(args)?,
                    self.before,
                )
                .await
            }
            FuncBackendKind::JsSchemaVariantDefinition => {
                FuncBackendJsSchemaVariantDefinition::create_and_execute(
                    self.func_dispatch_context,
                    &self.func,
                    &serde_json::Value::Null,
                    self.before,
                )
                .await
            }
            FuncBackendKind::Json => FuncBackendJson::create_and_execute(&self.args).await,
            FuncBackendKind::Array => FuncBackendArray::create_and_execute(&self.args).await,
            FuncBackendKind::Boolean => FuncBackendBoolean::create_and_execute(&self.args).await,
            FuncBackendKind::Identity => FuncBackendIdentity::create_and_execute(&self.args).await,
            FuncBackendKind::Diff => FuncBackendDiff::create_and_execute(&self.args).await,
            FuncBackendKind::Integer => FuncBackendInteger::create_and_execute(&self.args).await,
            FuncBackendKind::Map => FuncBackendMap::create_and_execute(&self.args).await,
            FuncBackendKind::Object => FuncBackendObject::create_and_execute(&self.args).await,
            FuncBackendKind::String => FuncBackendString::create_and_execute(&self.args).await,
            FuncBackendKind::Unset => Ok((None, None)),
            FuncBackendKind::Validation => {
                FuncBackendValidation::create_and_execute(
                    self.func_dispatch_context,
                    &self.func,
                    &self.args,
                    self.before,
                )
                .await
            }
            FuncBackendKind::JsValidation => {
                return Err(FuncRunnerError::DirectValidationFuncsNoLongerSupported(
                    self.func.id,
                ))
            }
            FuncBackendKind::JsAuthentication => {
                return Err(
                    FuncRunnerError::DirectAuthenticationFuncExecutionUnsupported(self.func.id),
                )
            }
        };

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

                let mut next_state_inner = Arc::unwrap_or_clone(running_state_func_run.clone());
                next_state_inner.set_state_to_post_processing();
                let next_state = Arc::new(next_state_inner);
                self.ctx
                    .layer_db()
                    .func_run()
                    .write(
                        next_state.clone(),
                        None,
                        self.ctx.events_tenancy(),
                        self.ctx.events_actor(),
                    )
                    .await?;
                let _ = self.result_tx.send(Ok(FuncRunValue::new(
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
                let mut next_state_inner = Arc::unwrap_or_clone(running_state_func_run.clone());
                next_state_inner.set_state_to_failure();
                let next_state = Arc::new(next_state_inner);
                self.ctx
                    .layer_db()
                    .func_run()
                    .write(
                        next_state.clone(),
                        None,
                        self.ctx.events_tenancy(),
                        self.ctx.events_actor(),
                    )
                    .await?;

                let _ = self.result_tx.send(Err(FuncRunnerError::ResultFailure {
                    kind,
                    message,
                    backend,
                }));
            }
            Err(err) => {
                let mut next_state_inner = Arc::unwrap_or_clone(running_state_func_run.clone());
                next_state_inner.set_state_to_failure();
                let next_state = Arc::new(next_state_inner);
                self.ctx
                    .layer_db()
                    .func_run()
                    .write(
                        next_state.clone(),
                        None,
                        self.ctx.events_tenancy(),
                        self.ctx.events_actor(),
                    )
                    .await?;
                let _ = self.result_tx.send(Err(err.into()));
            }
        }

        Ok(())
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
