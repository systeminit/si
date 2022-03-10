use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use telemetry::prelude::*;

use crate::{func::backend::FuncBackendResult, FuncBackendError};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendPropObjectArgs {
    pub value: Map<String, Value>,
}

impl FuncBackendPropObjectArgs {
    pub fn new(value: Map<String, Value>) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendPropObject {
    args: FuncBackendPropObjectArgs,
}

impl FuncBackendPropObject {
    pub fn new(args: FuncBackendPropObjectArgs) -> Self {
        Self { args }
    }

    #[instrument(
        name = "funcbackendpropobject.execute",
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

        if !value.is_object() {
            return Err(span.record_err(FuncBackendError::InvalidPropObjectData(value)));
        }

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok(value)
    }
}
