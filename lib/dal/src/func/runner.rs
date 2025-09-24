use std::{
    collections::VecDeque,
    sync::Arc,
};

use chrono::Utc;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json;
use si_events::{
    ActionId,
    ActionResultState,
    CasValue,
    ContentHash,
    EncryptedSecretKey,
    FuncRun,
    FuncRunBuilder,
    FuncRunBuilderError,
    FuncRunId,
    FuncRunLog,
    FuncRunLogId,
    FuncRunState,
    FuncRunValue,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{
    mpsc,
    oneshot,
};
use ulid::Ulid;
use veritech_client::{
    BeforeFunction,
    FunctionResult,
    FunctionResultFailure,
    FunctionResultFailureErrorKind,
    KillExecutionRequest,
    OutputStream,
    ResolverFunctionComponent,
    VeritechValueEncryptError,
    encrypt_value_tree,
};

use super::{
    backend::{
        FuncBackend,
        FuncDispatch,
        FuncDispatchContext,
        InvalidResolverFunctionTypeError,
        array::FuncBackendArray,
        boolean::FuncBackendBoolean,
        diff::FuncBackendDiff,
        float::FuncBackendFloat,
        identity::FuncBackendIdentity,
        integer::FuncBackendInteger,
        js_action::FuncBackendJsAction,
        js_attribute::{
            FuncBackendJsAttribute,
            FuncBackendJsAttributeArgs,
        },
        js_schema_variant_definition::FuncBackendJsSchemaVariantDefinition,
        json::FuncBackendJson,
        management::FuncBackendManagement,
        map::FuncBackendMap,
        normalize_to_array::FuncBackendNormalizeToArray,
        object::FuncBackendObject,
        resource_payload_to_value::FuncBackendResourcePayloadToValue,
        string::FuncBackendString,
        validation::FuncBackendValidation,
    },
    intrinsics::IntrinsicFunc,
};
use crate::{
    ActionPrototypeId,
    AttributeValue,
    AttributeValueId,
    ChangeSet,
    ChangeSetError,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    EncryptedSecret,
    Func,
    FuncBackendKind,
    FuncError,
    FuncId,
    KeyPairError,
    Prop,
    PropId,
    SchemaVariant,
    SchemaVariantError,
    Secret,
    SecretError,
    TransactionsError,
    WsEvent,
    WsEventError,
    WsEventResult,
    WsPayload,
    action::{
        Action,
        ActionError,
        prototype::{
            ActionPrototype,
            ActionPrototypeError,
        },
    },
    attribute::{
        prototype::argument::{
            AttributePrototypeArgument,
            AttributePrototypeArgumentError,
            AttributePrototypeArgumentId,
            value_source::ValueSource,
        },
        value::AttributeValueError,
    },
    func::backend::FuncBackendError,
    management::prototype::ManagementPrototypeId,
    prop::PropError,
    schema::variant::root_prop::RootPropChild,
    workspace::WorkspaceId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncRunnerError {
    #[error("action error: {0}")]
    ActionError(#[from] Box<ActionError>),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("before func missing expected code: {0}")]
    BeforeFuncMissingCode(FuncId),
    #[error("before func missing expected handler: {0}")]
    BeforeFuncMissingHandler(FuncId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error(
        "direct authentication func execution is unsupported (must go through \"before funcs\"), found: {0}"
    )]
    DirectAuthenticationFuncExecutionUnsupported(FuncId),
    #[error("direct validation funcs are no longer supported, found: {0}")]
    DirectValidationFuncsNoLongerSupported(FuncId),
    #[error("do not have permission to kill execution")]
    DoNotHavePermissionToKillExecution,
    #[error("empty widget options for secret prop id: {0}")]
    EmptyWidgetOptionsForSecretProp(PropId),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("function backend error: {0}")]
    FuncBackend(#[from] Box<FuncBackendError>),
    #[error("func run builder error: {0}")]
    FuncRunBuilder(#[from] FuncRunBuilderError),
    #[error("invalid resolver function type: {0}")]
    InvalidResolverFunctionType(#[from] InvalidResolverFunctionTypeError),
    #[error("kill execution failure: {0:?}")]
    KillExecutionFailure(FunctionResultFailure),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("missing attribute value for component ({0}) and prop ({1})")]
    MissingAttributeValue(ComponentId, PropId),
    #[error("no widget options for secret prop id: {0}")]
    NoWidgetOptionsForSecretProp(PropId),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("reconciliation funcs are no longer supported (found: {0})")]
    ReconciliationFuncsNoLongerSupported(FuncId),
    #[error("function run result failure: kind={kind}, message={message}, backend={backend}")]
    ResultFailure {
        kind: FunctionResultFailureErrorKind,
        message: String,
        backend: String,
    },
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("secret error: {0}")]
    Secret(#[from] Box<SecretError>),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error(
        "too many attribute prototype arguments for protoype corresponding to component ({0}) and prop ({1}): {2:?}"
    )]
    TooManyAttributePrototypeArguments(ComponentId, PropId, Vec<AttributePrototypeArgumentId>),
    #[error("too many attribute values for component ({0}) and prop ({1})")]
    TooManyAttributeValues(ComponentId, PropId),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error(
        "unexpected value source ({0:?}) for secret prop ({1}), attribute prototype argument ({2}) and component ({3})"
    )]
    UnexpectedValueSourceForSecretProp(
        ValueSource,
        PropId,
        AttributePrototypeArgumentId,
        ComponentId,
    ),
    #[error("veritech client error")]
    VeritechClient(#[from] veritech_client::ClientError),
    #[error("veritech value encrypt error: {0}")]
    VeritechValueEncrypt(#[from] VeritechValueEncryptError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<AttributePrototypeArgumentError> for FuncRunnerError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for FuncRunnerError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for FuncRunnerError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for FuncRunnerError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for FuncRunnerError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncBackendError> for FuncRunnerError {
    fn from(value: FuncBackendError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for FuncRunnerError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for FuncRunnerError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<SecretError> for FuncRunnerError {
    fn from(value: SecretError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for FuncRunnerError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for FuncRunnerError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

pub type FuncRunnerResult<T> = Result<T, FuncRunnerError>;

pub type FuncRunnerValueChannel = oneshot::Receiver<FuncRunnerResult<FuncRunValue>>;

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
            // job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = func.name.as_str(),
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.change_set.id = Empty,
            si.component.id = Empty,
            // si.func_run.func.args = Empty,
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
        let span = current_span_for_instrument_at!("debug");

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
            let (function_args_cas_address, _) = ctx.layer_db().cas().write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            let before = FuncRunner::before_funcs(ctx, component_id, &func).await?;

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
            // job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = func.name.as_str(),
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.action.id = Empty,
            si.action.kind = Empty,
            si.change_set.id = Empty,
            // si.func_run.func.args = Empty,
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
        let span = current_span_for_instrument_at!("debug");

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

            let (function_args_cas_address, _) = ctx.layer_db().cas().write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
                let code_json_value: serde_json::Value = code.clone().into();
                let code_cas_value: CasValue = code_json_value.into();
                let (hash, _) = ctx.layer_db().cas().write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )?;
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
            // job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = Empty,
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.attribute_value.id = Empty,
            si.change_set.id = Empty,
            si.component.id = Empty,
            // si.func_run.func.args = Empty,
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
        let span = current_span_for_instrument_at!("debug");

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
            let func = Func::get_by_id(ctx, func_id).await?;

            let args = serde_json::json!({
                "value": value,
                "validation_format": validation_format,
            });

            let function_args: CasValue = args.clone().into();

            let (function_args_cas_address, _) = ctx.layer_db().cas().write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
                let code_json_value: serde_json::Value = code.clone().into();
                let code_cas_value: CasValue = code_json_value.into();
                let (hash, _) = ctx.layer_db().cas().write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )?;
                hash
            } else {
                // Why are we doing this? Because the struct gods demand it. I have feelings.
                ContentHash::new("".as_bytes())
            };

            let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
            let component_name = Component::name_by_id(ctx, component_id).await?;

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
                .component_name(Some(component_name))
                .created_at(func_run_create_time)
                .updated_at(func_run_create_time)
                .build()?;

            if !span.is_disabled() {
                let mut id_buf = FuncRunId::array_to_str_buf();

                let id = func_run_inner.id().array_to_str(&mut id_buf);
                span.record("job.id", &id);
                span.record("si.func_run.id", &id);

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

        let runner = prepare(ctx, attribute_value_id, value, validation_format, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok(result_channel)
    }

    #[instrument(
        name = "func_runner.run_attribute_value",
        level = "info",
        skip_all,
        fields(
            job.id = Empty,
            // job.invoked_args = Empty,
            // job.instance = metadata.job_instance,
            job.invoked_name = Empty,
            // job.invoked_provider = metadata.job_invoked_provider,
            otel.kind = SpanKind::Producer.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.attribute_value.id = Empty,
            si.component.id = Empty,
            // si.func_run.func.args = Empty,
            si.func_run.func.backend_kind = Empty,
            si.func_run.func.backend_response_type = Empty,
            si.func_run.func.id = Empty,
            si.func_run.func.kind = Empty,
            si.func_run.func.name = Empty,
            si.func_run.id = Empty,
        )
    )]
    pub async fn run_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        func_id: FuncId,
        args: serde_json::Value,
    ) -> FuncRunnerResult<FuncRunnerValueChannel> {
        let span = current_span_for_instrument_at!("info");

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
            let func = Func::get_by_id(ctx, func_id).await?;

            let function_args: CasValue = args.clone().into();

            let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
            let component_name = Component::name_by_id(ctx, component_id).await?;
            let before = FuncRunner::before_funcs(ctx, component_id, &func).await?;

            let func_run_create_time = Utc::now();
            let mut func_run_builder = FuncRunBuilder::default();

            func_run_builder
                .actor(ctx.events_actor())
                .tenancy(ctx.events_tenancy())
                .backend_kind(func.backend_kind.into())
                .backend_response_type(func.backend_response_type.into())
                .function_name(func.name.clone())
                .function_kind(func.kind.into())
                .function_display_name(func.display_name.clone())
                .function_description(func.description.clone())
                .function_link(func.link.clone())
                .attribute_value_id(Some(attribute_value_id))
                .component_id(Some(component_id))
                .component_name(Some(component_name))
                .created_at(func_run_create_time)
                .updated_at(func_run_create_time);

            if !func.is_intrinsic() {
                let (function_args_cas_address, _) = ctx.layer_db().cas().write(
                    Arc::new(function_args.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )?;

                let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
                    let code_json_value: serde_json::Value = code.clone().into();
                    let code_cas_value: CasValue = code_json_value.into();
                    let (hash, _) = ctx.layer_db().cas().write(
                        Arc::new(code_cas_value.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )?;
                    hash
                } else {
                    ContentHash::new("".as_bytes())
                };

                func_run_builder.function_args_cas_address(function_args_cas_address);
                func_run_builder.function_code_cas_address(code_cas_hash);
            } else {
                // We could turn these into an option, except we postcard
                // serialize this data so we'd have to create a new type
                func_run_builder.function_args_cas_address(ContentHash::new("".as_bytes()));
                func_run_builder.function_code_cas_address(ContentHash::new("".as_bytes()));
            }

            let func_run_inner = func_run_builder.build()?;

            if !parent_span.is_disabled() {
                let mut id_buf = FuncRunId::array_to_str_buf();

                let id = func_run_inner.id().array_to_str(&mut id_buf);
                parent_span.record("job.id", &id);
                parent_span.record("si.func_run.id", &id);

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
                parent_span.record("si.component.id", component_id.array_to_str(&mut id_buf));
            }

            let func_run = Arc::new(func_run_inner);

            if !func.is_intrinsic() {
                ctx.layer_db()
                    .func_run()
                    .write(
                        func_run.clone(),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )
                    .await?;
            }

            Ok(FuncRunner {
                func_run,
                func,
                args,
                before,
            })
        }

        let runner = prepare(ctx, attribute_value_id, func_id, args, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok(result_channel)
    }

    #[instrument(
        name = "func_runner.run_management",
        level = "debug",
        skip_all,
        fields(
            job.id = Empty,
            // job.invoked_args = Empty,
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
            // si.func_run.func.args = Empty,
            si.func_run.func.backend_kind = Empty,
            si.func_run.func.backend_response_type = Empty,
            si.func_run.func.id = Empty,
            si.func_run.func.kind = Empty,
            si.func_run.func.name = Empty,
            si.func_run.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn run_management(
        ctx: &DalContext,
        prototype_id: ManagementPrototypeId,
        manager_component_id: ComponentId,
        management_func_id: FuncId,
        args: serde_json::Value,
    ) -> FuncRunnerResult<(FuncRunId, FuncRunnerValueChannel)> {
        let span = current_span_for_instrument_at!("debug");

        // Prepares the function for execution.
        //
        // Note: this function is internal so we can record early-returning errors in span metadata
        // and in order to time the function's preparation vs. execution timings.
        #[instrument(
            name = "func_runner.run_management.prepare",
            level = "debug",
            skip_all,
            fields()
        )]
        #[inline]
        async fn prepare(
            ctx: &DalContext,
            prototype_id: ManagementPrototypeId,
            manager_component_id: ComponentId,
            management_func_id: FuncId,
            args: serde_json::Value,
            span: &Span,
        ) -> FuncRunnerResult<FuncRunner> {
            let func = Func::get_by_id(ctx, management_func_id).await?;

            let function_args: CasValue = args.clone().into();
            let (function_args_cas_address, _) = ctx.layer_db().cas().write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
                let code_json_value: serde_json::Value = code.clone().into();
                let code_cas_value: CasValue = code_json_value.into();
                let (hash, _) = ctx.layer_db().cas().write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )?;
                hash
            } else {
                // Why are we doing this? Because the struct gods demand it. I have feelings.
                ContentHash::new("".as_bytes())
            };

            let before = FuncRunner::before_funcs(ctx, manager_component_id, &func).await?;
            let manager_component = Component::get_by_id(ctx, manager_component_id).await?;
            let component_name = manager_component.name(ctx).await?;
            let schema_name = manager_component.schema(ctx).await?.name;

            let change_set = ctx.change_set()?;

            let func_run_create_time = Utc::now();
            let func_run_inner = FuncRunBuilder::default()
                .actor(ctx.events_actor())
                .tenancy(ctx.events_tenancy())
                .backend_kind(func.backend_kind.into())
                .backend_response_type(func.backend_response_type.into())
                .function_name(func.name.clone())
                .function_kind(func.kind.into())
                .prototype_id(Some(prototype_id.into()))
                .function_display_name(func.display_name.clone())
                .function_description(func.description.clone())
                .function_link(func.link.clone())
                .function_args_cas_address(function_args_cas_address)
                .function_code_cas_address(code_cas_hash)
                .action_originating_change_set_id(Some(change_set.id))
                .action_originating_change_set_name(Some(change_set.name.to_owned()))
                .action_or_func_id(Some(func.id.into()))
                .attribute_value_id(None)
                .component_id(Some(manager_component_id))
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
                    "si.change_set.id",
                    func_run_inner.change_set_id().array_to_str(&mut id_buf),
                );
                span.record(
                    "si.component.id",
                    manager_component_id.array_to_str(&mut id_buf),
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
                func,
                args,
                before,
            })
        }

        let runner = prepare(
            ctx,
            prototype_id,
            manager_component_id,
            management_func_id,
            args,
            &span,
        )
        .await
        .map_err(|err| span.record_err(err))?;

        let func_run_id = runner.func_run.id();
        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok((func_run_id, result_channel))
    }

    pub async fn build_management(
        ctx: &DalContext,
        prototype_id: ManagementPrototypeId,
        manager_component_id: ComponentId,
        management_func_id: FuncId,
        args: serde_json::Value,
    ) -> FuncRunnerResult<FuncRunner> {
        let span = current_span_for_instrument_at!("debug");

        // Prepares the function for execution.
        //
        // Note: this function is internal so we can record early-returning errors in span metadata
        // and in order to time the function's preparation vs. execution timings.
        #[instrument(
            name = "func_runner.run_management.prepare",
            level = "debug",
            skip_all,
            fields()
        )]
        #[inline]
        async fn prepare(
            ctx: &DalContext,
            prototype_id: ManagementPrototypeId,
            manager_component_id: ComponentId,
            management_func_id: FuncId,
            args: serde_json::Value,
            span: &Span,
        ) -> FuncRunnerResult<FuncRunner> {
            let func = Func::get_by_id(ctx, management_func_id).await?;

            let function_args: CasValue = args.clone().into();
            let (function_args_cas_address, _) = ctx.layer_db().cas().write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
                let code_json_value: serde_json::Value = code.clone().into();
                let code_cas_value: CasValue = code_json_value.into();
                let (hash, _) = ctx.layer_db().cas().write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )?;
                hash
            } else {
                // Why are we doing this? Because the struct gods demand it. I have feelings.
                ContentHash::new("".as_bytes())
            };

            let before = FuncRunner::before_funcs(ctx, manager_component_id, &func).await?;
            let manager_component = Component::get_by_id(ctx, manager_component_id).await?;
            let component_name = manager_component.name(ctx).await?;
            let schema_name = manager_component.schema(ctx).await?.name;

            let change_set = ctx.change_set()?;

            let func_run_create_time = Utc::now();
            let func_run_inner = FuncRunBuilder::default()
                .actor(ctx.events_actor())
                .tenancy(ctx.events_tenancy())
                .backend_kind(func.backend_kind.into())
                .backend_response_type(func.backend_response_type.into())
                .function_name(func.name.clone())
                .function_kind(func.kind.into())
                .prototype_id(Some(prototype_id.into()))
                .function_display_name(func.display_name.clone())
                .function_description(func.description.clone())
                .function_link(func.link.clone())
                .function_args_cas_address(function_args_cas_address)
                .function_code_cas_address(code_cas_hash)
                .action_originating_change_set_id(Some(change_set.id))
                .action_originating_change_set_name(Some(change_set.name.to_owned()))
                .action_or_func_id(Some(func.id.into()))
                .attribute_value_id(None)
                .component_id(Some(manager_component_id))
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
                    "si.change_set.id",
                    func_run_inner.change_set_id().array_to_str(&mut id_buf),
                );
                span.record(
                    "si.component.id",
                    manager_component_id.array_to_str(&mut id_buf),
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
                func,
                args,
                before,
            })
        }

        let runner = prepare(
            ctx,
            prototype_id,
            manager_component_id,
            management_func_id,
            args,
            &span,
        )
        .await
        .map_err(|err| span.record_err(err))?;

        Ok(runner)
    }

    #[instrument(
        name = "func_runner.run_action",
        level = "info",
        skip_all,
        fields(
            job.id = Empty,
            // job.invoked_args = Empty,
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
            // si.func_run.func.args = Empty,
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
        let span = current_span_for_instrument_at!("debug");

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
            let func = Func::get_by_id(ctx, func_id).await?;
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
            let (function_args_cas_address, _) = ctx.layer_db().cas().write(
                Arc::new(function_args.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            let code_cas_hash = if let Some(code) = func.code_base64.as_ref() {
                let code_json_value: serde_json::Value = code.clone().into();
                let code_cas_value: CasValue = code_json_value.into();
                let (hash, _) = ctx.layer_db().cas().write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )?;
                hash
            } else {
                // Why are we doing this? Because the struct gods demand it. I have feelings.
                ContentHash::new("".as_bytes())
            };

            let before = FuncRunner::before_funcs(ctx, component_id, &func).await?;
            let component = Component::get_by_id(ctx, component_id).await?;
            let component_name = component.name(ctx).await?;
            let schema_name = component.schema(ctx).await?.name;

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
                .action_or_func_id(maybe_action_id.map(|a| a.into()))
                .prototype_id(Some(action_prototype_id.into()))
                .action_kind(Some(action_kind))
                .action_display_name(Some(prototype.name().clone()))
                .action_originating_change_set_id(maybe_action_originating_change_set_id)
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

        let runner = prepare(ctx, action_prototype_id, component_id, func_id, args, &span)
            .await
            .map_err(|err| span.record_err(err))?;

        let result_channel = runner.execute(ctx.clone(), span).await;

        Ok(result_channel)
    }

    #[instrument(
        name = "func_runner.kill_execution",
        level = "info",
        skip(ctx),
        fields(job.id = Empty, si.func_run.id = Empty)
    )]
    pub async fn kill_execution(ctx: &DalContext, func_run_id: FuncRunId) -> FuncRunnerResult<()> {
        let span = current_span_for_instrument_at!("info");

        if !span.is_disabled() {
            let mut id_buf = FuncRunId::array_to_str_buf();

            let id = func_run_id.array_to_str(&mut id_buf);
            span.record("job.id", &id);
            span.record("si.func_run.id", &id);
        }

        if !ctx.history_actor().email_is_systeminit(ctx).await? {
            return Err(FuncRunnerError::DoNotHavePermissionToKillExecution);
        }

        let result = ctx
            .veritech()
            .kill_execution(&KillExecutionRequest {
                execution_id: func_run_id.to_string(),
            })
            .await;

        match result? {
            FunctionResult::Success(_) => {
                info!(%func_run_id, "kill execution success");

                // NOTE(nick): why are we doing this here? Why aren't we returning a result channel? Well, Victor and I
                // did that originally, but we learned that most other func runner methods are abstracted out by
                // another dal entity. Those entities are responsible for "stamping" the func run to a terminating
                // state. For cancellation, there is no other entity. We need to do that here. Because of that, we also
                // need to know what the result of the cancellation request was. Therefore, this entire operation is
                // blocking and we do not return a result channel.
                FuncRunner::update_run(ctx, func_run_id, |func_run| {
                    func_run.set_state(FuncRunState::Killed)
                })
                .await?;

                // NOTE(nick): we may need to consider action result state as well as other fields on the func run
                // struct. This will require more testing and investigation. For now, I think what we have will
                // suffice... oh words, do not haunt me.
                Ok(())
            }
            FunctionResult::Failure(err) => Err(FuncRunnerError::KillExecutionFailure(err)),
        }
    }

    // Update the given func run in LayerDB, setting tenancy/actor to ctx.events_tenancy()/events_actor()).
    pub async fn update_run(
        ctx: &DalContext,
        id: FuncRunId,
        update_fn: impl FnOnce(&mut FuncRun),
    ) -> FuncRunnerResult<()> {
        let mut func_run = Arc::unwrap_or_clone(ctx.layer_db().func_run().try_read(id).await?);
        update_fn(&mut func_run);
        Ok(ctx
            .layer_db()
            .func_run()
            .write(
                Arc::new(func_run),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?)
    }

    // Update the latest func run for the given action in LayerDB, setting tenancy/actor to ctx.events_tenancy()/events_actor()).
    pub async fn update_run_for_action_id(
        ctx: &DalContext,
        action_id: ActionId,
        update_fn: impl FnOnce(&mut FuncRun),
    ) -> FuncRunnerResult<()> {
        let mut func_run = ctx
            .layer_db()
            .func_run()
            .get_last_run_for_action_id(ctx.events_tenancy().workspace_pk, action_id)
            .await?;
        update_fn(&mut func_run);
        Ok(ctx
            .layer_db()
            .func_run()
            .write(
                Arc::new(func_run),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?)
    }

    pub fn id(&self) -> FuncRunId {
        self.func_run.id()
    }

    pub async fn execute(
        self,
        ctx: DalContext,
        execution_parent_span: Span,
    ) -> FuncRunnerValueChannel {
        let func_run_id = self.func_run.id();
        let action_id = self.func_run.action_id();
        let (func_dispatch_context, output_stream_rx) = FuncDispatchContext::new(
            ctx.veritech().clone(),
            func_run_id,
            WorkspaceId::from(Ulid::from(self.func_run.workspace_pk())),
            self.func_run.change_set_id(),
        );
        let (result_tx, result_rx) = oneshot::channel();

        let logs_task = FuncRunnerLogsTask {
            ctx: ctx.clone(),
            func_run_id,
            output_stream_rx,
            action_id,
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
        func: &Func,
    ) -> FuncRunnerResult<Vec<BeforeFunction>> {
        if func.is_intrinsic() {
            // Intrinsic functions can't have before functions as they're never dispatched
            // to Veritech, so don't bother doing the expensive lookup to see what before
            // functions exist.
            return Ok(Vec::new());
        }

        let ordered_before_funcs_with_secret_keys =
            Self::ordered_before_funcs_with_secret_keys(ctx, component_id).await?;

        let mut before_functions = Vec::new();

        for (key, funcs) in ordered_before_funcs_with_secret_keys {
            let encrypted_secret = EncryptedSecret::get_by_key(ctx, key)
                .await?
                .ok_or(SecretError::EncryptedSecretNotFound(key))?;

            // Decrypt message from EncryptedSecret
            // Skip secret if unauthorized
            // Skip secret if we can't find keypair
            let decrypted_secret = match encrypted_secret.decrypt(ctx).await {
                Err(SecretError::KeyPair(KeyPairError::UnauthorizedKeyAccess))
                | Err(SecretError::KeyPair(KeyPairError::KeyPairNotFound(_))) => {
                    continue;
                }
                other_result => other_result,
            }?;

            let mut arg = decrypted_secret.message().into_inner();

            Self::inject_workspace_token(ctx, &mut arg).await?;

            // Re-encrypt raw Value for transmission to Veritech
            encrypt_value_tree(&mut arg, ctx.encryption_key())?;

            for func in funcs {
                before_functions.push(BeforeFunction {
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

        Ok(before_functions)
    }

    /// This _private_ method generates a flattened graph of before [`Funcs`](Func) with corresponding
    /// [`keys`](EncryptedSecretKey).
    #[instrument(
        name = "func_runner.before_funcs.ordered_before_funcs_with_secret_keys",
        level = "debug",
        skip_all
    )]
    async fn ordered_before_funcs_with_secret_keys(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> FuncRunnerResult<Vec<(EncryptedSecretKey, Vec<Func>)>> {
        let mut ordered_before_funcs_with_secret_keys = Vec::new();

        let mut work_queue = VecDeque::new();
        work_queue.push_back(component_id);

        while let Some(component_id) = work_queue.pop_front() {
            // First, collect all the children of "/root/secrets" for the given component.
            let secret_child_prop_ids = {
                let schema_variant = Component::schema_variant_id(ctx, component_id).await?;
                let secrets_prop = SchemaVariant::find_root_child_prop_id(
                    ctx,
                    schema_variant,
                    RootPropChild::Secrets,
                )
                .await?;
                Prop::direct_child_prop_ids_unordered(ctx, secrets_prop).await?
            };

            // Second, iterate through each child and descend or process based on its value source.
            for secret_child_prop_id in secret_child_prop_ids {
                let attribute_value_ids = Component::attribute_values_for_prop_id(
                    ctx,
                    component_id,
                    secret_child_prop_id,
                )
                .await?;
                if attribute_value_ids.len() > 1 {
                    return Err(FuncRunnerError::TooManyAttributeValues(
                        component_id,
                        secret_child_prop_id,
                    ));
                }
                let attribute_value_id =
                    *attribute_value_ids
                        .first()
                        .ok_or(FuncRunnerError::MissingAttributeValue(
                            component_id,
                            secret_child_prop_id,
                        ))?;

                // We have the attribute value for the secret prop and component, so now we need to find the attribute
                // prototype argument id for the corresponding prototype in order to find the value source.
                let attribute_prototype_id =
                    AttributeValue::prototype_id(ctx, attribute_value_id).await?;
                let attribute_prototype_argument_ids =
                    AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
                        .await?;
                if attribute_prototype_argument_ids.len() > 1 {
                    return Err(FuncRunnerError::TooManyAttributePrototypeArguments(
                        component_id,
                        secret_child_prop_id,
                        attribute_prototype_argument_ids.clone(),
                    ));
                }

                // If there's not attribute prototype argument yet, the prototype is either using "si:unset" or nothing
                // has been connected to us.
                let attribute_prototype_argument_id = match attribute_prototype_argument_ids.first()
                {
                    Some(attribute_prototype_argument_id) => *attribute_prototype_argument_id,
                    None => continue,
                };

                match AttributePrototypeArgument::value_source(ctx, attribute_prototype_argument_id)
                    .await?
                {
                    // I am not the secret defining component, but have a subscription, so let's find where it comes from
                    ValueSource::ValueSubscription(input) => {
                        // get the component for this av:
                        let component_id =
                            AttributeValue::component_id(ctx, input.attribute_value_id).await?;
                        work_queue.push_back(component_id);
                    }
                    // I found the secret defining component, let's grab the before funcs!
                    ValueSource::Secret(_) => {
                        let auth_funcs =
                            Self::auth_funcs_for_secret_child_prop_id(ctx, secret_child_prop_id)
                                .await?;
                        let attribute_value =
                            AttributeValue::get_by_id(ctx, attribute_value_id).await?;
                        let maybe_value = attribute_value.value(ctx).await?;

                        // NOTE(nick): in the future, we could likely run auth funcs without secret inputs without
                        // this check. As it is written here, we only load the other funcs if a secret is populated.
                        if let Some(value) = maybe_value {
                            let key = Secret::key_from_value_in_attribute_value(value)?;
                            ordered_before_funcs_with_secret_keys.push((key, auth_funcs));
                        }
                    }
                    // There is no such thing as a socket connection, so no value can come from here.
                    ValueSource::InputSocket(_) => {}
                    value_source => {
                        return Err(FuncRunnerError::UnexpectedValueSourceForSecretProp(
                            value_source,
                            secret_child_prop_id,
                            attribute_prototype_argument_id,
                            component_id,
                        ));
                    }
                }
            }
        }

        // Reverse the order of secrets in which they were found.
        ordered_before_funcs_with_secret_keys.reverse();

        Ok(ordered_before_funcs_with_secret_keys)
    }

    async fn inject_workspace_token(
        ctx: &DalContext,
        value: &mut serde_json::Value,
    ) -> FuncRunnerResult<()> {
        if let Some(token) = ctx.get_workspace_token().await? {
            if let serde_json::Value::Object(obj) = value {
                obj.insert("WorkspaceToken".to_string(), token.into());
            }
        }
        Ok(())
    }

    /// This _private_ method gathers the authentication functions for a given [`PropId`](Prop)
    /// underneath "/root/secrets" (e.g. "/root/secret/MySecretDefinitionName").
    #[instrument(
        name = "func_runner.before_funcs.auth_funcs_for_secret_prop_id",
        level = "debug",
        skip_all
    )]
    async fn auth_funcs_for_secret_child_prop_id(
        ctx: &DalContext,
        secret_child_prop_id: PropId,
    ) -> FuncRunnerResult<Vec<Func>> {
        let secret_child_prop = Prop::get_by_id(ctx, secret_child_prop_id).await?;

        let secret_definition_name = secret_child_prop
            .widget_options
            .ok_or(FuncRunnerError::NoWidgetOptionsForSecretProp(
                secret_child_prop_id,
            ))?
            .pop()
            .ok_or(FuncRunnerError::EmptyWidgetOptionsForSecretProp(
                secret_child_prop_id,
            ))?
            .value;

        // Iterate through all default secret defining schema variants and find the output socket that matches the
        // provided secret child prop. This works on two assumptions. First: secret defining schema variants can have
        // one and only one output socket, and that socket must correspond to the secret that it defines. Second:
        // secret definition names are unique with the change set.
        let mut auth_funcs = Vec::new();
        for secret_defining_schema_variant_id in
            SchemaVariant::list_default_secret_defining_ids(ctx).await?
        {
            let secret_output_socket = SchemaVariant::find_output_socket_for_secret_defining_id(
                ctx,
                secret_defining_schema_variant_id,
            )
            .await?;
            if secret_output_socket.name() == secret_definition_name {
                for auth_func_id in
                    SchemaVariant::list_auth_func_ids_for_id(ctx, secret_defining_schema_variant_id)
                        .await?
                {
                    auth_funcs.push(Func::get_by_id(ctx, auth_func_id).await?)
                }
                return Ok(auth_funcs);
            }
        }

        Ok(auth_funcs)
    }
}

struct FuncRunnerLogsTask {
    ctx: DalContext,
    func_run_id: FuncRunId,
    output_stream_rx: mpsc::Receiver<OutputStream>,
    action_id: Option<ActionId>,
}

impl FuncRunnerLogsTask {
    const NAME: &'static str = "Dal::FuncRunnerLogsTask";

    async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(
                si.error.message = ?err,
                task = Self::NAME,
                "error while processing function logs"
            );
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

            WsEvent::func_run_log_updated(
                &self.ctx,
                func_run_log.func_run_id(),
                func_run_log.id(),
                self.action_id,
            )
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
        let span = current_span_for_instrument_at!("debug");

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
            error!(
                si.error.message = ?err,
                task = Self::NAME,
                "error while dispatching and running function"
            );
        }
    }

    async fn try_run(self) -> FuncRunnerResult<()> {
        if !self.func.is_intrinsic() {
            FuncRunner::update_run(&self.ctx, self.func_run.id(), |func_run| {
                func_run.set_state(FuncRunState::Running);
            })
            .await?;
        }

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
            FuncBackendKind::JsAttribute => {
                // NOTE(nick): changing the behavior is not great because the spans will imply that
                // a different form of execution ran vs. what actually happened. Why do this here
                // then? This is the last possible moment before the function executes. Not only
                // that, but import logic will migrate from the older func to the newer one, so this
                // will hopefully not be exercised often... and eventually never.
                match IntrinsicFunc::maybe_from_str(self.func.name.as_str()) {
                    Some(IntrinsicFunc::ResourcePayloadToValue) => {
                        info!(
                            si.func_run.id = %self.func_run.id(),
                            si.func_run.func.id = %self.func.id,
                            si.func_run.func.name = %self.func.name,
                            si.func_run.func.backend_kind = %self.func_run.backend_kind(),
                            "ignoring JsAttribute func backend kind for ResourcePayloadToValue intrinsic"
                        );
                        FuncBackendResourcePayloadToValue::create_and_execute(&self.args).await
                    }
                    Some(IntrinsicFunc::NormalizeToArray) => {
                        info!(
                            si.func_run.id = %self.func_run.id(),
                            si.func_run.func.id = %self.func.id,
                            si.func_run.func.name = %self.func.name,
                            si.func_run.func.backend_kind = %self.func_run.backend_kind(),
                            "ignoring JsAttribute func backend kind for NormalizeToArray intrinsic"
                        );
                        FuncBackendNormalizeToArray::create_and_execute(&self.args).await
                    }
                    Some(_) | None => {
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
                }
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
            FuncBackendKind::Float => FuncBackendFloat::create_and_execute(&self.args).await,
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
            FuncBackendKind::JsReconciliation => {
                return Err(FuncRunnerError::ReconciliationFuncsNoLongerSupported(
                    self.func.id,
                ));
            }
            FuncBackendKind::JsValidation => {
                return Err(FuncRunnerError::DirectValidationFuncsNoLongerSupported(
                    self.func.id,
                ));
            }
            FuncBackendKind::JsAuthentication => {
                return Err(
                    FuncRunnerError::DirectAuthenticationFuncExecutionUnsupported(self.func.id),
                );
            }
            FuncBackendKind::Management => {
                FuncBackendManagement::create_and_execute(
                    self.func_dispatch_context,
                    &self.func,
                    &self.args,
                    self.before,
                )
                .await
            }
            FuncBackendKind::ResourcePayloadToValue => {
                FuncBackendResourcePayloadToValue::create_and_execute(&self.args).await
            }
            FuncBackendKind::NormalizeToArray => {
                FuncBackendNormalizeToArray::create_and_execute(&self.args).await
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

                if !self.func.is_intrinsic() {
                    FuncRunner::update_run(&self.ctx, self.func_run.id(), |func_run| {
                        func_run.set_state(FuncRunState::PostProcessing);
                    })
                    .await?;
                }

                // Don't stop running the function just because we can't send the result
                #[allow(unused_must_use)]
                self.result_tx.send(Ok(FuncRunValue::new(
                    self.func_run.id(),
                    unprocessed_value,
                    value,
                )));
            }
            Err(FuncBackendError::ResultFailure {
                kind,
                message,
                backend,
            }) => {
                if !self.func.is_intrinsic() {
                    FuncRunner::update_run(&self.ctx, self.func_run.id(), |func_run| {
                        func_run.set_state(FuncRunState::Failure);
                    })
                    .await?;
                }

                // Don't stop running the function just because we can't send the result
                #[allow(unused_must_use)]
                self.result_tx.send(Err(FuncRunnerError::ResultFailure {
                    kind,
                    message,
                    backend,
                }));
            }
            Err(err) => {
                if !self.func.is_intrinsic() {
                    FuncRunner::update_run(&self.ctx, self.func_run.id(), |func_run| {
                        func_run.set_state(FuncRunState::Failure);
                    })
                    .await?;
                }

                // Don't stop running the function just because we can't send the result
                #[allow(unused_must_use)]
                self.result_tx.send(Err(err.into()));
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
    action_id: Option<ActionId>,
}

impl WsEvent {
    pub async fn func_run_log_updated(
        ctx: &DalContext,
        func_run_id: FuncRunId,
        func_run_log_id: FuncRunLogId,
        action_id: Option<ActionId>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncRunLogUpdated(FuncRunLogUpdatedPayload {
                func_run_id,
                func_run_log_id,
                action_id,
            }),
        )
        .await
    }
}
