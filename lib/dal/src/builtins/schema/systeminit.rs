use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{
    schema::SchemaVariant, socket::SocketArity, BuiltinsResult, DalContext, DiagramKind,
    ExternalProvider, SchemaKind, StandardModel,
};

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    system(ctx).await?;
    Ok(())
}

async fn system(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "system".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::System).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };
    schema.set_ui_hidden(ctx, true).await?;

    let (schema_variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;

    schema_variant.finalize(ctx).await?;

    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
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
