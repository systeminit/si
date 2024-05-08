use telemetry::prelude::*;

use crate::func::authoring::{FuncAuthoringResult, TestExecuteFuncResult};
use crate::func::backend::FuncDispatchContext;
use crate::func::binding::critical_section::execute_critical_section;
use crate::func::binding::{FuncBindingError, LogLinePayload};
use crate::secret::before_funcs_for_component;
use crate::{ComponentId, DalContext, Func, WsEvent, WsEventResult};

#[instrument(
    name = "func.authoring.test_execute_func.perform_test_execution",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn perform_test_execution(
    ctx: &DalContext,
    func: Func,
    args: serde_json::Value,
    execution_key: String,
    component_id: ComponentId,
) -> FuncAuthoringResult<TestExecuteFuncResult> {
    let before = before_funcs_for_component(ctx, component_id).await?;

    // Create a new dispatch context. We'll use this for both live logs and for the actual execution.
    let (context, mut rx) = FuncDispatchContext::new(ctx);

    let cached_execution_key = execution_key.clone();
    let (func_id, inner_ctx) = (func.id, ctx.clone());

    // Publish live logs back to the frontend.
    let log_handler = tokio::spawn(async move {
        let (ctx, mut output) = (&inner_ctx, Vec::new());

        while let Some(output_stream) = rx.recv().await {
            output.push(output_stream.clone());

            let log_line = LogLinePayload {
                stream: output_stream,
                func_id,
                execution_key: execution_key.clone(),
            };
            publish_immediately(ctx, WsEvent::log_line(ctx, log_line).await?).await?;
        }
        Ok::<_, FuncBindingError>(output)
    });

    // Perform the execution and compile all the logs.
    let (value, _unprocessed_value) =
        execute_critical_section(func, &args, context, before).await?;
    let logs = log_handler.await??;

    Ok(TestExecuteFuncResult {
        id: func_id,
        args,
        execution_key: cached_execution_key,
        output: value.unwrap_or(serde_json::Value::Null),
        logs,
    })
}

// TODO(nick): we do not want these floating around the codebase. Let's sync with Fletcher and
// come up with a better plan to avoid "YOLO publish immediately" calls unless sandboxed.
async fn publish_immediately(ctx: &DalContext, ws_event: WsEvent) -> WsEventResult<()> {
    let subject = format!("si.workspace_pk.{}.event", ws_event.workspace_pk());
    let msg_bytes = serde_json::to_vec(&ws_event)?;
    ctx.nats_conn().publish(subject, msg_bytes.into()).await?;
    Ok(())
}
