use base64::engine::general_purpose;
use base64::Engine;

use crate::func::authoring::{DummyExecutionResult, FuncAuthoringResult};
use crate::func::binding::FuncBinding;
use crate::{ComponentId, DalContext, Func, FuncId};

pub(crate) async fn dummy_execute_func(
    ctx: &DalContext,
    id: FuncId,
    args: serde_json::Value,
    execution_key: String,
    code: String,
    component_id: ComponentId,
) -> FuncAuthoringResult<DummyExecutionResult> {
    let func = Func::get_by_id_or_error(ctx, id).await?;

    // Cache the old code.
    let cached_code = func.code_base64.to_owned();

    // Modify the func, but do not commit it!
    Func::modify_by_id(ctx, func.id, |func| {
        func.code_base64 = Some(general_purpose::STANDARD_NO_PAD.encode(code));
        Ok(())
    })
    .await?;

    // Execute the func, but do not commit the results!
    let (output, logs) = FuncBinding::execute_dummy(
        ctx,
        id,
        func.backend_kind,
        args.clone(),
        execution_key.clone(),
        component_id,
    )
    .await?;

    // Restore the old code. We should not need to do this, but this is failsafe in case someone
    // commits the func changes by accident.
    Func::modify_by_id(ctx, func.id, |func| {
        func.code_base64 = cached_code;
        Ok(())
    })
    .await?;

    Ok(DummyExecutionResult {
        id,
        args,
        execution_key,
        output,
        logs,
    })
}
