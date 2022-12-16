use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech_client::{
    Client as VeritechClient, CommandRunResultSuccess, FunctionResult, OutputStream,
    ResolverFunctionResponseType,
};

use crate::{label_list::ToLabelList, DalContext, Func, FuncId, PropKind, StandardModel};

pub mod array;
pub mod boolean;
pub mod identity;
pub mod integer;
pub mod js_attribute;
pub mod js_command;
pub mod js_confirmation;
pub mod js_workflow;
pub mod map;
pub mod prop_object;
pub mod string;

#[derive(Error, Debug)]
pub enum FuncBackendError {
    #[error("invalid data - expected a valid array entry value, got: {0}")]
    InvalidArrayEntryData(serde_json::Value),
    #[error("expected same array entry prop kinds - expected {0}, found: {1}")]
    DifferingArrayEntryPropKinds(PropKind, PropKind),
    #[error("result failure: kind={kind}, message={message}, backend={backend}")]
    ResultFailure {
        kind: String,
        message: String,
        backend: String,
    },
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("invalid data - got unset when not expected")]
    UnexpectedUnset,
    #[error("veritech client error: {0}")]
    VeritechClient(#[from] veritech_client::ClientError),
    #[error("send error")]
    SendError,
    #[error("dispatch func missing code_base64 {0}")]
    DispatchMissingBase64(FuncId),
    #[error("dispatch func missing handler {0}")]
    DispatchMissingHandler(FuncId),
    #[error("function result command run error: {0:?}")]
    FunctionResultCommandRun(FunctionResult<CommandRunResultSuccess>),
}

pub type FuncBackendResult<T> = Result<T, FuncBackendError>;

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
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    JsAttribute,
    JsWorkflow,
    JsConfirmation,
    JsCommand,
    Map,
    PropObject,
    String,
    Unset,
    Json,
}

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
    Array,
    Boolean,
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    Map,
    PropObject,
    Qualification,
    CodeGeneration,
    Confirmation,
    String,
    Unset,
    Json,
    Validation,
    Workflow,
    Command,
}

impl From<ResolverFunctionResponseType> for FuncBackendResponseType {
    fn from(value: ResolverFunctionResponseType) -> Self {
        match value {
            ResolverFunctionResponseType::Array => FuncBackendResponseType::Array,
            ResolverFunctionResponseType::Boolean => FuncBackendResponseType::Boolean,
            ResolverFunctionResponseType::Identity => FuncBackendResponseType::Identity,
            ResolverFunctionResponseType::Integer => FuncBackendResponseType::Integer,
            ResolverFunctionResponseType::Map => FuncBackendResponseType::Map,
            ResolverFunctionResponseType::PropObject => FuncBackendResponseType::PropObject,
            ResolverFunctionResponseType::Qualification => FuncBackendResponseType::Qualification,
            ResolverFunctionResponseType::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            ResolverFunctionResponseType::Confirmation => FuncBackendResponseType::Confirmation,
            ResolverFunctionResponseType::String => FuncBackendResponseType::String,
            ResolverFunctionResponseType::Unset => FuncBackendResponseType::Unset,
            ResolverFunctionResponseType::Json => FuncBackendResponseType::Json,
            ResolverFunctionResponseType::Validation => FuncBackendResponseType::Validation,
            ResolverFunctionResponseType::Command => FuncBackendResponseType::Command,
            ResolverFunctionResponseType::Workflow => FuncBackendResponseType::Workflow,
        }
    }
}

impl From<FuncBackendResponseType> for ResolverFunctionResponseType {
    fn from(value: FuncBackendResponseType) -> Self {
        match value {
            FuncBackendResponseType::Array => ResolverFunctionResponseType::Array,
            FuncBackendResponseType::Boolean => ResolverFunctionResponseType::Boolean,
            FuncBackendResponseType::Integer => ResolverFunctionResponseType::Integer,
            FuncBackendResponseType::Identity => ResolverFunctionResponseType::Identity,
            FuncBackendResponseType::Map => ResolverFunctionResponseType::Map,
            FuncBackendResponseType::PropObject => ResolverFunctionResponseType::PropObject,
            FuncBackendResponseType::Qualification => ResolverFunctionResponseType::Qualification,
            FuncBackendResponseType::CodeGeneration => ResolverFunctionResponseType::CodeGeneration,
            FuncBackendResponseType::Confirmation => ResolverFunctionResponseType::Confirmation,
            FuncBackendResponseType::String => ResolverFunctionResponseType::String,
            FuncBackendResponseType::Unset => ResolverFunctionResponseType::Unset,
            FuncBackendResponseType::Json => ResolverFunctionResponseType::Json,
            FuncBackendResponseType::Validation => ResolverFunctionResponseType::Validation,
            FuncBackendResponseType::Command => ResolverFunctionResponseType::Command,
            FuncBackendResponseType::Workflow => ResolverFunctionResponseType::Workflow,
        }
    }
}

impl ToLabelList for FuncBackendKind {}

#[derive(Debug, Clone)]
pub struct FuncDispatchContext {
    pub veritech: VeritechClient,
    pub output_tx: mpsc::Sender<OutputStream>,
}

impl FuncDispatchContext {
    pub fn new(ctx: &DalContext) -> (Self, mpsc::Receiver<OutputStream>) {
        let (output_tx, rx) = mpsc::channel(64);
        (
            Self {
                veritech: ctx.veritech().clone(),
                output_tx,
            },
            rx,
        )
    }

    pub fn into_inner(self) -> (VeritechClient, mpsc::Sender<OutputStream>) {
        (self.veritech, self.output_tx)
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
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)>
    where
        <Self::Output as ExtractPayload>::Payload: Serialize,
    {
        let executor = Self::create(context, func, args)?;
        Ok(executor.execute().await?)
    }

    /// This private function creates the "request" to send to veritech in a shape that it
    /// likes. The request's type is [`Self`].
    fn create(
        context: FuncDispatchContext,
        func: &Func,
        args: &serde_json::Value,
    ) -> FuncBackendResult<Box<Self>> {
        let args = Self::Args::deserialize(args)?;
        let code_base64 = func
            .code_base64()
            .ok_or_else(|| FuncBackendError::DispatchMissingBase64(*func.id()))?;
        let handler = func
            .handler()
            .ok_or_else(|| FuncBackendError::DispatchMissingHandler(*func.id()))?;
        let value = Self::new(context, code_base64, handler, args);
        Ok(value)
    }

    // TODO: re-enable encryption
    //{
    //    for view in value.extract()? {
    //        ComponentView::reencrypt_secrets(ctx, view).await?;
    //    }
    //}

    #[instrument(
    name = "funcdispatch.execute",
    skip_all,
    level = "debug",
    fields(
    otel.kind = %FormattedSpanKind(SpanKind::Client),
    otel.status_code = Empty,
    otel.status_message = Empty,
    si.func.result = Empty
    )
    )]
    async fn execute(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)>
    where
        <Self::Output as ExtractPayload>::Payload: Serialize,
    {
        let span = Span::current();

        // NOTE(nick,wendy): why is a debug output of "self" a valid backend?
        let backend = format!("{:?}", &self);
        let value = match self.dispatch().await.map_err(|err| span.record_err(err))? {
            FunctionResult::Success(check_result) => {
                let payload = serde_json::to_value(check_result.extract()?)?;
                (Some(payload.clone()), Some(payload))
            }
            FunctionResult::Failure(failure) => {
                return Err(span.record_err(FuncBackendError::ResultFailure {
                    kind: failure.error.kind,
                    backend,
                    message: failure.error.message,
                }));
            }
        };

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok(value)
    }

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
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
    skip_all,
    level = "debug",
    fields(
    otel.kind = %FormattedSpanKind(SpanKind::Client),
    otel.status_code = Empty,
    otel.status_message = Empty,
    si.func.result = Empty
    )
    )]
    async fn execute(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let span = Span::current();

        let value = self.inline().await?;

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
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
