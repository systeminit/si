use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use telemetry::prelude::*;

use crate::{func::backend::FuncBackendResult, FuncBackendError};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendMapArgs {
    pub value: Map<String, Value>,
}

impl FuncBackendMapArgs {
    pub fn new(value: Map<String, Value>) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendMap {
    args: FuncBackendMapArgs,
}

impl FuncBackendMap {
    pub fn new(args: FuncBackendMapArgs) -> Self {
        Self { args }
    }

    #[instrument(
        name = "funcbackendmap.execute",
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
            return Err(span.record_err(FuncBackendError::InvalidMapData(value)));
        }

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok(value)
    }
}
