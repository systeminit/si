use crate::{
    BuiltinsResult, DalContext, Func, SchemaError, StandardModel, WorkflowPrototype,
    WorkflowPrototypeContext,
};

pub async fn migrate(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    poem(ctx).await?;
    Ok(())
}

async fn poem(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let func_name = "si:poem";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
    let title = "Lero Lero";

    let context = WorkflowPrototypeContext::default(); // workspace level
    if WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await?
        .is_empty()
    {
        WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title).await?;
    }
    Ok(())
}
