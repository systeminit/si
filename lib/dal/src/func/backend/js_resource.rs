use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use tokio::sync::mpsc;
use veritech::{Client, FunctionResult, OutputStream, ResourceSyncRequest};

use crate::func::backend::{FuncBackendError, FuncBackendResult};
use crate::ComponentView;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsResourceSyncArgs {
    pub component: ComponentView,
}

#[derive(Debug)]
pub struct FuncBackendJsResourceSync {
    veritech: Client,
    output_tx: mpsc::Sender<OutputStream>,
    request: ResourceSyncRequest,
}

impl FuncBackendJsResourceSync {
    pub fn new(
        veritech: Client,
        output_tx: mpsc::Sender<OutputStream>,
        handler: impl Into<String>,
        args: FuncBackendJsResourceSyncArgs,
        code_base64: impl Into<String>,
    ) -> Self {
        let request = ResourceSyncRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "rodrigosantoro".to_string(),
            handler: handler.into(),
            component: args.component.into(),
            code_base64: code_base64.into(),
        };

        Self {
            veritech,
            output_tx,
            request,
        }
    }

    #[instrument(
    name = "funcbackendjsresourcesync.execute",
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
            .execute_resource_sync(self.output_tx, &self.request)
            .await
            .map_err(|err| span.record_err(err))?;
        let value = match result {
            FunctionResult::Success(result) => serde_json::to_value(&result)?,
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
