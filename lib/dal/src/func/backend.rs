use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech::{FunctionResult, OutputStream};

use crate::{label_list::ToLabelList, DalContext, Func, FuncId, PropKind, StandardModel};

pub mod array;
pub mod boolean;
pub mod identity;
pub mod integer;
pub mod js_attribute;
pub mod js_code_generation;
pub mod js_command;
pub mod js_qualification;
pub mod js_resource;
pub mod js_workflow;
pub mod map;
pub mod prop_object;
pub mod string;
pub mod validation;

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
    VeritechClient(#[from] veritech::ClientError),
    #[error("send error")]
    SendError,
    #[error("dispatch func missing code_base64 {0}")]
    DispatchMissingBase64(FuncId),
    #[error("dispatch func missing code_base64 {0}")]
    DispatchMissingHandler(FuncId),
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
    JsQualification,
    JsResourceSync,
    JsCodeGeneration,
    JsAttribute,
    JsWorkflow,
    JsCommand,
    Map,
    PropObject,
    String,
    Unset,
    // Commented out while we climb back up - Adam & Fletcher
    //Number (Float?),
    //EmptyObject,
    //EmptyArray,
    Json,
    //Js,
    ValidateStringValue,
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
    ResourceSync,
    CodeGeneration,
    String,
    Unset,
    Json,
    Validation,
    Workflow,
    Command,
}

impl From<FuncBackendKind> for FuncBackendResponseType {
    fn from(backend_kind: FuncBackendKind) -> Self {
        match backend_kind {
            FuncBackendKind::Array => FuncBackendResponseType::Array,
            FuncBackendKind::Boolean => FuncBackendResponseType::Boolean,
            FuncBackendKind::Identity => FuncBackendResponseType::Identity,
            FuncBackendKind::Integer => FuncBackendResponseType::Integer,
            FuncBackendKind::JsQualification => FuncBackendResponseType::Qualification,
            FuncBackendKind::JsResourceSync => FuncBackendResponseType::ResourceSync,
            FuncBackendKind::JsCodeGeneration => FuncBackendResponseType::CodeGeneration,
            FuncBackendKind::JsAttribute => FuncBackendResponseType::String,
            FuncBackendKind::JsWorkflow => FuncBackendResponseType::Workflow,
            FuncBackendKind::JsCommand => FuncBackendResponseType::Command,
            FuncBackendKind::Map => FuncBackendResponseType::Map,
            FuncBackendKind::PropObject => FuncBackendResponseType::PropObject,
            FuncBackendKind::String => FuncBackendResponseType::String,
            FuncBackendKind::Unset => FuncBackendResponseType::Unset,
            FuncBackendKind::Json => FuncBackendResponseType::Json,
            FuncBackendKind::ValidateStringValue => FuncBackendResponseType::Validation,
        }
    }
}

impl ToLabelList for FuncBackendKind {}

#[derive(Debug, Clone)]
pub struct FuncDispatchContext {
    pub veritech: veritech::Client,
    pub output_tx: mpsc::Sender<OutputStream>,
}

impl FuncDispatchContext {
    pub fn new(ctx: &DalContext<'_, '_, '_>) -> (Self, mpsc::Receiver<OutputStream>) {
        let (output_tx, rx) = mpsc::channel(64);
        (
            Self {
                veritech: ctx.veritech().clone(),
                output_tx,
            },
            rx,
        )
    }

    pub fn into_inner(self) -> (veritech::Client, mpsc::Sender<OutputStream>) {
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
    //    for view in value.extract() {
    //        ComponentView::reencrypt_secrets(ctx, view).await?;
    //    }
    //}

    #[instrument(
        name = "funcdispatch.execute",
        skip_all,
        level = "debug",
        fields(
            otel.kind = %SpanKind::Client,
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

        let backend = format!("{:?}", &self);
        let value = match self.dispatch().await.map_err(|err| span.record_err(err))? {
            FunctionResult::Success(check_result) => {
                let payload = serde_json::to_value(check_result.extract())?;
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
            otel.kind = %SpanKind::Client,
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

    fn extract(self) -> Self::Payload;
}
