use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use si_events::{
    ChangeSetId,
    FuncRunId,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech_client::{
    ActionRunResultSuccess,
    BeforeFunction,
    Client as VeritechClient,
    FunctionResult,
    FunctionResultFailureErrorKind,
    OutputStream,
    ResolverFunctionResponseType,
};

use crate::{
    Func,
    FuncId,
    PropKind,
    label_list::ToLabelList,
    workspace::WorkspaceId,
};

pub mod array;
pub mod boolean;
pub mod debug;
pub mod diff;
pub mod float;
pub mod identity;
pub mod integer;
pub mod js_action;
pub mod js_attribute;
pub mod js_schema_variant_definition;
pub mod json;
pub mod management;
pub mod map;
pub mod normalize_to_array;
pub mod object;
pub mod resource_payload_to_value;
pub mod string;
pub mod validation;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncBackendError {
    #[error("expected same array entry prop kinds - expected {0}, found: {1}")]
    DifferingArrayEntryPropKinds(PropKind, PropKind),
    #[error("dispatch func missing code_base64 {0}")]
    DispatchMissingBase64(FuncId),
    #[error("dispatch func missing handler {0}")]
    DispatchMissingHandler(FuncId),
    #[error("function result action run error: {0:?}")]
    FunctionResultActionRun(FunctionResult<Box<ActionRunResultSuccess>>),
    #[error("invalid data - expected a valid array entry value, got: {0}")]
    InvalidArrayEntryData(serde_json::Value),
    #[error("result failure: kind={kind}, message={message}, backend={backend}")]
    ResultFailure {
        kind: FunctionResultFailureErrorKind,
        message: String,
        backend: String,
    },
    #[error("send error")]
    SendError,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("unable to decode ulid")]
    Ulid(#[from] ulid::DecodeError),
    #[error("veritech client error: {0}")]
    VeritechClient(#[from] Box<veritech_client::ClientError>),
}

pub type FuncBackendResult<T> = Result<T, FuncBackendError>;

impl From<veritech_client::ClientError> for FuncBackendError {
    fn from(value: veritech_client::ClientError) -> Self {
        Box::new(value).into()
    }
}

// NOTE(nick,zack): do not add "remain::sorted" for postcard de/ser. We need the order to be
// retained.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
pub enum FuncBackendKind {
    Array,
    Boolean,
    /// Comparison between two JSON values
    Diff,
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    JsAction,
    JsAttribute,
    JsAuthentication,
    Json,
    // NOTE(nick): this has been deprecated. Not adding serde deprecated tag in case it affects the type.
    JsReconciliation,
    JsSchemaVariantDefinition,
    JsValidation,
    Map,
    Object,
    String,
    Unset,
    Validation,
    Management,
    ResourcePayloadToValue,
    NormalizeToArray,
    Float,
    Debug,
}

impl From<FuncBackendKind> for si_events::FuncBackendKind {
    fn from(value: FuncBackendKind) -> Self {
        match value {
            FuncBackendKind::Array => si_events::FuncBackendKind::Array,
            FuncBackendKind::Boolean => si_events::FuncBackendKind::Boolean,
            FuncBackendKind::Diff => si_events::FuncBackendKind::Diff,
            FuncBackendKind::Identity => si_events::FuncBackendKind::Identity,
            FuncBackendKind::Integer => si_events::FuncBackendKind::Integer,
            FuncBackendKind::Float => si_events::FuncBackendKind::Float,
            FuncBackendKind::JsAction => si_events::FuncBackendKind::JsAction,
            FuncBackendKind::JsAttribute => si_events::FuncBackendKind::JsAttribute,
            FuncBackendKind::JsAuthentication => si_events::FuncBackendKind::JsAuthentication,
            FuncBackendKind::Json => si_events::FuncBackendKind::Json,
            FuncBackendKind::JsReconciliation => si_events::FuncBackendKind::JsReconciliation,
            FuncBackendKind::JsSchemaVariantDefinition => {
                si_events::FuncBackendKind::JsSchemaVariantDefinition
            }
            FuncBackendKind::JsValidation => si_events::FuncBackendKind::JsValidation,
            FuncBackendKind::Map => si_events::FuncBackendKind::Map,
            FuncBackendKind::Object => si_events::FuncBackendKind::Object,
            FuncBackendKind::String => si_events::FuncBackendKind::String,
            FuncBackendKind::Unset => si_events::FuncBackendKind::Unset,
            FuncBackendKind::Validation => si_events::FuncBackendKind::Validation,
            FuncBackendKind::Management => si_events::FuncBackendKind::Management,
            FuncBackendKind::ResourcePayloadToValue => {
                si_events::FuncBackendKind::ResourcePayloadToValue
            }
            FuncBackendKind::NormalizeToArray => si_events::FuncBackendKind::NormalizeToArray,
            FuncBackendKind::Debug => si_events::FuncBackendKind::Debug,
        }
    }
}

impl From<si_events::FuncBackendKind> for FuncBackendKind {
    fn from(value: si_events::FuncBackendKind) -> Self {
        match value {
            si_events::FuncBackendKind::Array => FuncBackendKind::Array,
            si_events::FuncBackendKind::Boolean => FuncBackendKind::Boolean,
            si_events::FuncBackendKind::Diff => FuncBackendKind::Diff,
            si_events::FuncBackendKind::Float => FuncBackendKind::Float,
            si_events::FuncBackendKind::Identity => FuncBackendKind::Identity,
            si_events::FuncBackendKind::Integer => FuncBackendKind::Integer,
            si_events::FuncBackendKind::JsAction => FuncBackendKind::JsAction,
            si_events::FuncBackendKind::JsAttribute => FuncBackendKind::JsAttribute,
            si_events::FuncBackendKind::JsAuthentication => FuncBackendKind::JsAuthentication,
            si_events::FuncBackendKind::Json => FuncBackendKind::Json,
            si_events::FuncBackendKind::JsReconciliation => FuncBackendKind::JsReconciliation,
            si_events::FuncBackendKind::JsSchemaVariantDefinition => {
                FuncBackendKind::JsSchemaVariantDefinition
            }
            si_events::FuncBackendKind::JsValidation => FuncBackendKind::JsValidation,
            si_events::FuncBackendKind::Map => FuncBackendKind::Map,
            si_events::FuncBackendKind::Object => FuncBackendKind::Object,
            si_events::FuncBackendKind::String => FuncBackendKind::String,
            si_events::FuncBackendKind::Unset => FuncBackendKind::Unset,
            si_events::FuncBackendKind::Validation => FuncBackendKind::Validation,
            si_events::FuncBackendKind::Management => FuncBackendKind::Management,
            si_events::FuncBackendKind::ResourcePayloadToValue => {
                FuncBackendKind::ResourcePayloadToValue
            }
            si_events::FuncBackendKind::NormalizeToArray => FuncBackendKind::NormalizeToArray,
            si_events::FuncBackendKind::Debug => FuncBackendKind::Debug,
        }
    }
}

// NOTE(nick,zack): do not add "remain::sorted" for postcard de/ser. We need the order to be
// retained.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
pub enum FuncBackendResponseType {
    Action,
    Array,
    Boolean,
    CodeGeneration,
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    Json,
    Map,
    Object,
    Qualification,
    // NOTE(nick): this has been deprecated. Not adding serde deprecated tag in case it affects the type.
    Reconciliation,
    SchemaVariantDefinition,
    String,
    Unset,
    Validation,
    Void,
    Management,
    Float,
    Debug,
}

impl From<FuncBackendResponseType> for si_events::FuncBackendResponseType {
    fn from(value: FuncBackendResponseType) -> Self {
        match value {
            FuncBackendResponseType::Action => si_events::FuncBackendResponseType::Action,
            FuncBackendResponseType::Array => si_events::FuncBackendResponseType::Array,
            FuncBackendResponseType::Boolean => si_events::FuncBackendResponseType::Boolean,
            FuncBackendResponseType::CodeGeneration => {
                si_events::FuncBackendResponseType::CodeGeneration
            }
            FuncBackendResponseType::Float => si_events::FuncBackendResponseType::Float,
            FuncBackendResponseType::Identity => si_events::FuncBackendResponseType::Identity,
            FuncBackendResponseType::Integer => si_events::FuncBackendResponseType::Integer,
            FuncBackendResponseType::Json => si_events::FuncBackendResponseType::Json,
            FuncBackendResponseType::Map => si_events::FuncBackendResponseType::Map,
            FuncBackendResponseType::Object => si_events::FuncBackendResponseType::Object,
            FuncBackendResponseType::Qualification => {
                si_events::FuncBackendResponseType::Qualification
            }
            FuncBackendResponseType::Reconciliation => {
                si_events::FuncBackendResponseType::Reconciliation
            }
            FuncBackendResponseType::SchemaVariantDefinition => {
                si_events::FuncBackendResponseType::SchemaVariantDefinition
            }
            FuncBackendResponseType::String => si_events::FuncBackendResponseType::String,
            FuncBackendResponseType::Unset => si_events::FuncBackendResponseType::Unset,
            FuncBackendResponseType::Validation => si_events::FuncBackendResponseType::Validation,
            FuncBackendResponseType::Void => si_events::FuncBackendResponseType::Void,
            FuncBackendResponseType::Management => si_events::FuncBackendResponseType::Management,
            FuncBackendResponseType::Debug => si_events::FuncBackendResponseType::Debug,
        }
    }
}

impl From<si_events::FuncBackendResponseType> for FuncBackendResponseType {
    fn from(value: si_events::FuncBackendResponseType) -> Self {
        match value {
            si_events::FuncBackendResponseType::Action => FuncBackendResponseType::Action,
            si_events::FuncBackendResponseType::Array => FuncBackendResponseType::Array,
            si_events::FuncBackendResponseType::Boolean => FuncBackendResponseType::Boolean,
            si_events::FuncBackendResponseType::CodeGeneration => {
                FuncBackendResponseType::CodeGeneration
            }
            si_events::FuncBackendResponseType::Float => FuncBackendResponseType::Float,
            si_events::FuncBackendResponseType::Identity => FuncBackendResponseType::Identity,
            si_events::FuncBackendResponseType::Integer => FuncBackendResponseType::Integer,
            si_events::FuncBackendResponseType::Json => FuncBackendResponseType::Json,
            si_events::FuncBackendResponseType::Map => FuncBackendResponseType::Map,
            si_events::FuncBackendResponseType::Object => FuncBackendResponseType::Object,
            si_events::FuncBackendResponseType::Qualification => {
                FuncBackendResponseType::Qualification
            }
            si_events::FuncBackendResponseType::Reconciliation => {
                FuncBackendResponseType::Reconciliation
            }
            si_events::FuncBackendResponseType::SchemaVariantDefinition => {
                FuncBackendResponseType::SchemaVariantDefinition
            }
            si_events::FuncBackendResponseType::String => FuncBackendResponseType::String,
            si_events::FuncBackendResponseType::Unset => FuncBackendResponseType::Unset,
            si_events::FuncBackendResponseType::Validation => FuncBackendResponseType::Validation,
            si_events::FuncBackendResponseType::Void => FuncBackendResponseType::Void,
            si_events::FuncBackendResponseType::Management => FuncBackendResponseType::Management,
            si_events::FuncBackendResponseType::Debug => FuncBackendResponseType::Debug,
        }
    }
}

impl From<ResolverFunctionResponseType> for FuncBackendResponseType {
    fn from(value: ResolverFunctionResponseType) -> Self {
        match value {
            ResolverFunctionResponseType::Action => FuncBackendResponseType::Action,
            ResolverFunctionResponseType::Array => FuncBackendResponseType::Array,
            ResolverFunctionResponseType::Boolean => FuncBackendResponseType::Boolean,
            ResolverFunctionResponseType::Float => FuncBackendResponseType::Float,
            ResolverFunctionResponseType::Identity => FuncBackendResponseType::Identity,
            ResolverFunctionResponseType::Integer => FuncBackendResponseType::Integer,
            ResolverFunctionResponseType::Map => FuncBackendResponseType::Map,
            ResolverFunctionResponseType::Object => FuncBackendResponseType::Object,
            ResolverFunctionResponseType::Qualification => FuncBackendResponseType::Qualification,
            ResolverFunctionResponseType::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            ResolverFunctionResponseType::String => FuncBackendResponseType::String,
            ResolverFunctionResponseType::Unset => FuncBackendResponseType::Unset,
            ResolverFunctionResponseType::Json => FuncBackendResponseType::Json,
            ResolverFunctionResponseType::Void => FuncBackendResponseType::Void,
            ResolverFunctionResponseType::Management => FuncBackendResponseType::Management,
            ResolverFunctionResponseType::Debug => FuncBackendResponseType::Debug,
        }
    }
}

#[derive(Error, Debug)]
#[error("invalid resolver function type: {0}")]
pub struct InvalidResolverFunctionTypeError(FuncBackendResponseType);

impl TryFrom<FuncBackendResponseType> for ResolverFunctionResponseType {
    type Error = InvalidResolverFunctionTypeError;

    fn try_from(value: FuncBackendResponseType) -> Result<Self, Self::Error> {
        let value = match &value {
            FuncBackendResponseType::Action => ResolverFunctionResponseType::Action,
            FuncBackendResponseType::Array => ResolverFunctionResponseType::Array,
            FuncBackendResponseType::Boolean => ResolverFunctionResponseType::Boolean,
            FuncBackendResponseType::Float => ResolverFunctionResponseType::Float,
            FuncBackendResponseType::Integer => ResolverFunctionResponseType::Integer,
            FuncBackendResponseType::Identity => ResolverFunctionResponseType::Identity,
            FuncBackendResponseType::Map => ResolverFunctionResponseType::Map,
            FuncBackendResponseType::Object => ResolverFunctionResponseType::Object,
            FuncBackendResponseType::Qualification => ResolverFunctionResponseType::Qualification,
            FuncBackendResponseType::CodeGeneration => ResolverFunctionResponseType::CodeGeneration,
            FuncBackendResponseType::String => ResolverFunctionResponseType::String,
            FuncBackendResponseType::Unset => ResolverFunctionResponseType::Unset,
            FuncBackendResponseType::Json => ResolverFunctionResponseType::Json,
            FuncBackendResponseType::Validation => {
                return Err(InvalidResolverFunctionTypeError(value));
            }
            FuncBackendResponseType::Reconciliation => {
                return Err(InvalidResolverFunctionTypeError(value));
            }
            FuncBackendResponseType::SchemaVariantDefinition => {
                return Err(InvalidResolverFunctionTypeError(value));
            }
            FuncBackendResponseType::Void => ResolverFunctionResponseType::Void,
            FuncBackendResponseType::Management => ResolverFunctionResponseType::Management,
            FuncBackendResponseType::Debug => ResolverFunctionResponseType::Debug,
        };
        Ok(value)
    }
}

impl ToLabelList for FuncBackendKind {}

#[derive(Debug, Clone)]
pub struct FuncDispatchContext {
    pub veritech: VeritechClient,
    pub output_tx: mpsc::Sender<OutputStream>,
    pub func_run_id: FuncRunId,
    pub workspace_id: WorkspaceId,
    pub change_set_id: ChangeSetId,
}

impl FuncDispatchContext {
    pub fn new(
        veritech_client: VeritechClient,
        func_run_id: FuncRunId,
        workspace_id: WorkspaceId,
        change_set_id: ChangeSetId,
    ) -> (Self, mpsc::Receiver<OutputStream>) {
        let (output_tx, rx) = mpsc::channel(64);
        (
            Self {
                veritech: veritech_client,
                output_tx,
                func_run_id,
                workspace_id,
                change_set_id,
            },
            rx,
        )
    }

    pub fn into_inner(
        self,
    ) -> (
        VeritechClient,
        mpsc::Sender<OutputStream>,
        WorkspaceId,
        ChangeSetId,
    ) {
        (
            self.veritech,
            self.output_tx,
            self.workspace_id,
            self.change_set_id,
        )
    }
}

#[async_trait]
pub trait FuncDispatch: std::fmt::Debug {
    type Args: DeserializeOwned + Send + std::fmt::Debug;
    type Output: ExtractPayload + std::fmt::Debug;

    async fn create_and_execute(
        context: FuncDispatchContext,
        func: &Func,
        args: &serde_json::Value,
        before: Vec<BeforeFunction>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)>
    where
        <Self::Output as ExtractPayload>::Payload: Serialize,
    {
        let executor = Self::create(context, func, args, before)?;
        Ok(executor.execute().await?)
    }

    /// This private function creates the "request" to send to veritech in a shape that it
    /// likes. The request's type is [`Self`].
    fn create(
        context: FuncDispatchContext,
        func: &Func,
        args: &serde_json::Value,
        before: Vec<BeforeFunction>,
    ) -> FuncBackendResult<Box<Self>> {
        let args = Self::Args::deserialize(args)?;
        let code_base64 = func
            .code_base64
            .as_deref()
            .ok_or_else(|| FuncBackendError::DispatchMissingBase64(func.id))?;
        let handler = func
            .handler
            .as_deref()
            .ok_or_else(|| FuncBackendError::DispatchMissingHandler(func.id))?;
        let value = Self::new(context, code_base64, handler, args, before);
        Ok(value)
    }

    #[instrument(
        name = "funcdispatch.execute",
        level = "debug",
        skip_all,
        fields(
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.func.result = Empty,
        )
    )]
    async fn execute(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)>
    where
        <Self::Output as ExtractPayload>::Payload: Serialize,
    {
        let span = current_span_for_instrument_at!("debug");

        // NOTE(nick,wendy): why is a debug output of "self" a valid backend?
        let backend = format!("{:?}", &self);
        let value = match self.dispatch().await.map_err(|err| span.record_err(err))? {
            FunctionResult::Success(check_result) => {
                let payload = serde_json::to_value(check_result.extract()?)?;
                (Some(payload.clone()), Some(payload))
            }
            FunctionResult::Failure(failure) => {
                return Err(span.record_err(FuncBackendError::ResultFailure {
                    kind: failure.error().kind.to_owned(),
                    backend,
                    message: failure.error().message.to_owned(),
                }));
            }
        };

        span.record_ok();
        span.record("si.func.result", tracing::field::debug(&value));
        Ok(value)
    }

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
        before: Vec<BeforeFunction>,
    ) -> Box<Self>;
    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>>;
}

#[async_trait]
pub trait FuncBackend {
    type Args: DeserializeOwned + Send + std::fmt::Debug;

    async fn create_and_execute(
        args: &serde_json::Value,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let executor = Self::create(args)?;
        Ok(executor.execute().await?)
    }

    fn create(args: &serde_json::Value) -> FuncBackendResult<Box<Self>> {
        let args = Self::Args::deserialize(args)?;
        Ok(Self::new(args))
    }

    #[instrument(
        name = "funcbackend.execute",
        level = "debug",
        skip_all,
        fields(
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.func.result = Empty,
        )
    )]
    async fn execute(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let span = current_span_for_instrument_at!("debug");

        let value = self.inline().await?;

        span.record_ok();
        span.record("si.func.result", tracing::field::debug(&value));
        Ok(value)
    }

    fn new(args: Self::Args) -> Box<Self>;
    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)>;
}

pub trait ExtractPayload {
    type Payload: std::fmt::Debug;

    fn extract(self) -> FuncBackendResult<Self::Payload>;
}
