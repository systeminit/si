use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{func::backend::FuncBackendResult, FuncBackendError};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendBooleanArgs {
    pub value: bool,
}

impl FuncBackendBooleanArgs {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendBoolean {
    args: FuncBackendBooleanArgs,
}

impl FuncBackendBoolean {
    pub fn new(args: FuncBackendBooleanArgs) -> Self {
        Self { args }
    }

    #[instrument(
    name = "funcbackendboolean.execute",
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

        if !value.is_boolean() {
            return Err(span.record_err(FuncBackendError::InvalidBooleanData(value)));
        }

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok(value)
    }
}
