use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{func::backend::FuncBackendResult, FuncBackendError};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendIntegerArgs {
    pub value: i64,
}

impl FuncBackendIntegerArgs {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendInteger {
    args: FuncBackendIntegerArgs,
}

impl FuncBackendInteger {
    pub fn new(args: FuncBackendIntegerArgs) -> Self {
        Self { args }
    }

    #[instrument(
        name = "funcbackendinteger.execute",
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

        if !value.is_i64() {
            return Err(span.record_err(FuncBackendError::InvalidIntegerData(value)));
        }

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok(value)
    }
}
