use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::component::ComponentKind;
use crate::{
    socket::SocketArity, BuiltinsResult, DalContext, DiagramKind, ExternalProvider, SchemaKind,
    StandardModel,
};

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    system(ctx).await?;
    Ok(())
}

async fn system(ctx: &DalContext) -> BuiltinsResult<()> {
    let (mut schema, schema_variant, _) = BuiltinSchemaHelpers::create_schema_and_variant(
        ctx,
        "system",
        SchemaKind::System,
        ComponentKind::Standard,
        None,
    )
    .await?;

    schema.set_ui_hidden(ctx, true).await?;
    schema_variant.finalize(ctx).await?;

    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id, _) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
    let (_component_output_provider, _component_output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "component_output",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;

    Ok(())
}
