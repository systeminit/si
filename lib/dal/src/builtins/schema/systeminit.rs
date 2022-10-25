use crate::builtins::schema::{BuiltinSchemaHelpers, MigrationDriver};
use crate::component::ComponentKind;
use crate::{
    socket::SocketArity, BuiltinsError, BuiltinsResult, DalContext, DiagramKind, ExternalProvider,
    SchemaKind, StandardModel,
};

pub async fn migrate(ctx: &DalContext, driver: &MigrationDriver) -> BuiltinsResult<()> {
    system(ctx, driver).await?;
    Ok(())
}

async fn system(ctx: &DalContext, driver: &MigrationDriver) -> BuiltinsResult<()> {
    let (mut schema, schema_variant, _) = match BuiltinSchemaHelpers::create_schema_and_variant(
        ctx,
        "system",
        SchemaKind::System,
        ComponentKind::Standard,
        None,
    )
    .await?
    {
        Some(tuple) => tuple,
        None => return Ok(()),
    };

    schema.set_ui_hidden(ctx, true).await?;
    schema_variant.finalize(ctx).await?;

    let identity_func_item = driver
        .get_func_item("si:identity")
        .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

    let (_component_output_provider, _component_output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "component_output",
        None,
        identity_func_item.func_id,
        identity_func_item.func_binding_id,
        identity_func_item.func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;

    Ok(())
}
