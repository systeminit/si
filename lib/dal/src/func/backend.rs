use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech::{Client, FunctionResult, OutputStream, ResolverFunctionRequest};

use crate::{edit_field::ToSelectWidget, label_list::ToLabelList};

pub mod validation;

#[derive(Error, Debug)]
pub enum FuncBackendError {
    #[error("invalid data - expected a string, got: {0}")]
    InvalidStringData(serde_json::Value),
    #[error("result failure: kind={kind}, message={message}")]
    ResultFailure { kind: String, message: String },
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("invalid data - got unset when not expected")]
    UnexpectedUnset,
    #[error("veritech client error: {0}")]
    VeritechClient(#[from] veritech::ClientError),
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
    String,
    // Commented out while we climb back up - Adam & Fletcher
    //Number,
    //Boolean,
    //Object,
    //Array,
    //EmptyObject,
    //EmptyArray,
    Unset,
    //Json,
    //Js,
    JsString,
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
    String,
    // Commented out while we climb back up - Adam & Fletcher
    //Number,
    //Boolean,
    //Object,
    //Array,
    Unset,
    //Json,
    Validation,
}

impl ToLabelList for FuncBackendKind {}
impl ToSelectWidget for FuncBackendKind {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendStringArgs {
    pub value: String,
}

impl FuncBackendStringArgs {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendString {
    args: FuncBackendStringArgs,
}

impl FuncBackendString {
    pub fn new(args: FuncBackendStringArgs) -> Self {
        Self { args }
    }

    #[instrument(
        name = "funcbackendstring.execute",
        skip_all,
        level = "debug",
        fields(
            otel.kind = %SpanKind::Client,
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.func.result = Empty
        )
    )]
    pub async fn execute(self) -> FuncBackendResult<serde_json::Value> {
        let span = Span::current();

        let value = serde_json::to_value(&self.args.value)?;
        // You can be damn sure this is a string, really - because
        // the inner type there is a string. But hey - better safe
        // than sorry!
        if !value.is_string() {
            return Err(span.record_err(FuncBackendError::InvalidStringData(value)));
        }

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok(value)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendJsStringArgs {
    pub arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct FuncBackendJsString {
    veritech: Client,
    output_tx: mpsc::Sender<OutputStream>,
    request: ResolverFunctionRequest,
}

impl FuncBackendJsString {
    pub fn new(
        veritech: Client,
        output_tx: mpsc::Sender<OutputStream>,
        handler: impl Into<String>,
        args: HashMap<String, serde_json::Value>,
        code_base64: impl Into<String>,
    ) -> Self {
        let request = ResolverFunctionRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "tomcruise".to_string(),
            handler: handler.into(),
            parameters: Some(args),
            code_base64: code_base64.into(),
        };

        Self {
            veritech,
            output_tx,
            request,
        }
    }

    #[instrument(
        name = "funcbackendjsstring.execute",
        skip_all,
        level = "debug",
        fields(
            otel.kind = %SpanKind::Client,
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.func.result = Empty
        )
    )]
    pub async fn execute(self) -> FuncBackendResult<serde_json::Value> {
        let span = Span::current();

        let result = self
            .veritech
            .execute_resolver_function("veritech.function.resolver", self.output_tx, &self.request)
            .await
            .map_err(|err| span.record_err(err))?;
        let value = match result {
            FunctionResult::Success(success) => {
                if success.unset {
                    return Err(span.record_err(FuncBackendError::UnexpectedUnset));
                }
                if !success.data.is_string() {
                    return Err(span.record_err(FuncBackendError::InvalidStringData(success.data)));
                }
                success.data
            }
            FunctionResult::Failure(failure) => {
                return Err(span.record_err(FuncBackendError::ResultFailure {
                    kind: failure.error.kind,
                    message: failure.error.message,
                }));
            }
        };

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok(value)
    }
}
