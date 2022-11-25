use crate::builtins::schema::{BuiltinSchemaHelpers, MigrationDriver};
use crate::component::ComponentKind;
use crate::validation::Validation;
use crate::{
    schema::SchemaUiMenu, AttributeContext, BuiltinsError, BuiltinsResult, DalContext, DiagramKind,
    InternalProvider, PropKind, SchemaError, SchemaKind, SocketArity, StandardModel,
};

const FRAME_NODE_COLOR: i64 = 0xFFFFFF;

pub async fn migrate(ctx: &DalContext, driver: &MigrationDriver) -> BuiltinsResult<()> {
    generic_frame(ctx, driver).await?;
    Ok(())
}

async fn generic_frame(ctx: &DalContext, driver: &MigrationDriver) -> BuiltinsResult<()> {
    let (schema, schema_variant, root_prop) = match BuiltinSchemaHelpers::create_schema_and_variant(
        ctx,
        "Generic Frame",
        SchemaKind::Configuration,
        ComponentKind::Standard,
        Some(FRAME_NODE_COLOR),
    )
    .await?
    {
        Some(tuple) => tuple,
        None => return Ok(()),
    };

    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "Generic Frame", "Frames", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop and validation creation
    let color_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "Color",
        PropKind::String,
        None,
        Some(root_prop.si_prop_id),
        None,
    )
    .await?;

    // Sockets
    let identity_func_item = driver
        .get_func_item("si:identity")
        .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

    let (_docker_hub_credential_explicit_internal_provider, _input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema_variant.id(),
            "Frame",
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;

    driver
        .create_validation(
            ctx,
            Validation::StringIsHexColor { value: None },
            *color_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

    schema_variant.finalize(ctx).await?;

    Ok(())
}
