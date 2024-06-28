use crate::{DalContext, FuncId, SchemaVariant, SchemaVariantId};

use super::{
    FuncBinding, FuncBindingDiscriminants, FuncBindings, FuncBindingsError, FuncBindingsResult,
};

pub(crate) async fn assemble_auth_bindings(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncBindingsResult<Vec<FuncBinding>> {
    let schema_variant_ids = SchemaVariant::list_for_auth_func(ctx, func_id).await?;
    let mut bindings = vec![];
    for schema_variant_id in schema_variant_ids {
        bindings.push(FuncBinding::Authentication {
            schema_variant_id,
            func_id,
        });
    }
    Ok(bindings)
}
pub async fn create_auth_binding(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> FuncBindingsResult<FuncBindings> {
    SchemaVariant::new_authentication_prototype(ctx, func_id, schema_variant_id).await?;
    let updated_bindings = FuncBindings::from_func_id(ctx, func_id).await?;
    Ok(updated_bindings)
}

pub async fn delete_auth_binding(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> FuncBindingsResult<FuncBindings> {
    SchemaVariant::remove_authentication_prototype(ctx, func_id, schema_variant_id).await?;
    let updated_bindings = FuncBindings::from_func_id(ctx, func_id).await?;

    Ok(updated_bindings)
}
