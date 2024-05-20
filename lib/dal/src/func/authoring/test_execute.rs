use telemetry::prelude::*;

use crate::func::authoring::{FuncAuthoringResult, TestExecuteFuncResult};
use crate::func::runner::FuncRunner;
use crate::{ComponentId, DalContext, Func};

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
    let (func_run_id, _result_channel) =
        FuncRunner::run_test(ctx, func, args, component_id).await?;

    Ok(TestExecuteFuncResult { func_run_id })
}
