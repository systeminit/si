use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{
    schema::SchemaVariant, socket::SocketArity, BuiltinsResult, DalContext, DiagramKind,
    ExternalProvider, SchemaKind, StandardModel,
};

pub async fn migrate(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
    system(ctx).await?;
    Ok(())
}

async fn system(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
    let name = "system".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::System).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };
    schema.set_ui_hidden(ctx, true).await?;

    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;

    variant.finalize(ctx).await?;

    let identity_func = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
    let (_component_output_provider, _component_output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *variant.id(),
        "component_output",
        None,
        identity_func.0,
        identity_func.1,
        identity_func.2,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;

    Ok(())
}
