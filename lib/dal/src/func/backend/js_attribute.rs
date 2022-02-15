use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use tokio::sync::mpsc;
use veritech::{Client, FunctionResult, OutputStream, ResolverFunctionRequest};

use crate::func::backend::{FuncBackendError, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendJsAttributeArgs {
    pub component: veritech::ResolverFunctionComponent,
}

#[derive(Debug)]
pub struct FuncBackendJsAttribute {
    veritech: Client,
    output_tx: mpsc::Sender<OutputStream>,
    request: ResolverFunctionRequest,
}

impl FuncBackendJsAttribute {
    pub fn new(
        veritech: Client,
        output_tx: mpsc::Sender<OutputStream>,
        handler: impl Into<String>,
        args: FuncBackendJsAttributeArgs,
        code_base64: impl Into<String>,
    ) -> Self {
        let request = ResolverFunctionRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "tomcruise".to_string(),
            handler: handler.into(),
            component: args.component,
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
            .execute_resolver_function(self.output_tx, &self.request)
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
