use crate::{
    BuiltinsResult, DalContext, Func, SchemaError, StandardModel, WorkflowPrototype,
    WorkflowPrototypeContext,
};

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    poem(ctx).await?;
    failure(ctx).await?;
    ctx.blocking_commit().await?;
    Ok(())
}

async fn poem(ctx: &DalContext) -> BuiltinsResult<()> {
    let func_name = "si:poemWorkflow";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
    let title = "Lero Lero";

    let context = WorkflowPrototypeContext::default();
    if WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await?
        .is_empty()
    {
        WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title).await?;
    }
    Ok(())
}

async fn failure(ctx: &DalContext) -> BuiltinsResult<()> {
    let func_name = "si:failureWorkflow";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
    let title = "Epic Fail!";

    let context = WorkflowPrototypeContext::default(); // workspace level
    if WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await?
        .is_empty()
    {
        WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title).await?;
    }
    Ok(())
}
