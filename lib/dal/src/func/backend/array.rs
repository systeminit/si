use serde::{Deserialize, Serialize};
use serde_json::Value;
use telemetry::prelude::*;

use crate::{func::backend::FuncBackendResult, FuncBackendError, PropKind};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendArrayArgs {
    pub value: Vec<Value>,
}

impl FuncBackendArrayArgs {
    pub fn new(value: Vec<Value>) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendArray {
    args: FuncBackendArrayArgs,
}

impl FuncBackendArray {
    pub fn new(args: FuncBackendArrayArgs) -> Self {
        Self { args }
    }

    #[instrument(
    name = "funcbackendarray.execute",
    skip_all,
    level = "debug",
    fields(
    otel.kind = %SpanKind::Client,
    otel.status_code = Empty,
    otel.status_message = Empty,
    si.func.result = Empty
    )
    )]
    pub async fn execute(self) -> FuncBackendResult<(serde_json::Value, serde_json::Value)> {
        let span = Span::current();

        let value = serde_json::to_value(&self.args.value)?;

        // Ensure each entry of the array is valid and is of the same prop kind.
        if let Some(array) = value.as_array() {
            let mut first_kind_found: Option<PropKind> = None;
            for entry in array {
                let entry_kind = if entry.is_array() {
                    PropKind::Array
                } else if entry.is_i64() {
                    PropKind::Integer
                } else if entry.is_object() {
                    PropKind::Object
                } else if entry.is_boolean() {
                    PropKind::Boolean
                } else if entry.is_string() {
                    PropKind::String
                } else {
                    return Err(
                        span.record_err(FuncBackendError::InvalidArrayEntryData(value.clone()))
                    );
                };

                if let Some(v) = &first_kind_found {
                    if v != &entry_kind {
                        return Err(span.record_err(
                            FuncBackendError::DifferingArrayEntryPropKinds(*v, entry_kind),
                        ));
                    }
                } else {
                    first_kind_found = Some(entry_kind);
                }
            }
        } else {
            return Err(span.record_err(FuncBackendError::InvalidArrayData(value)));
        }

        span.record_ok();
        span.record("si.func.result", &tracing::field::debug(&value));
        Ok((value, serde_json::json!([])))
    }
}
