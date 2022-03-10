use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::func::backend::{FuncBackendError, FuncBackendResult};

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
