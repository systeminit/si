use crate::builtins::schema::BuiltinSchemaHelpers;

use crate::socket::{SocketArity, SocketEdgeKind, SocketKind};
use crate::{
    qualification_prototype::QualificationPrototypeContext,
    schema::{SchemaVariant, UiMenu},
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    BuiltinsError, BuiltinsResult, DalContext, DiagramKind, Func, FuncError, InternalProvider,
    PropKind, QualificationPrototype, SchemaError, SchemaKind, Socket, StandardModel,
};

const BUTANE_DOCS_FCOS_1_4_URL: &str = "https://coreos.github.io/butane/config-fcos-v1_4/";

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    butane(ctx).await?;
    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`Butane`](https://coreos.github.io/butane/).
async fn butane(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "butane".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup. The variant color was taken from the coreos logo.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(0x5590cc)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("butane")).await?;
    ui_menu.set_category(ctx, Some("coreos".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop creation
    let variant_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "variant",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
    )
    .await?;
    let version_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "version",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
    )
    .await?;
    let systemd_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "systemd",
        PropKind::Object,
        Some(root_prop.domain_prop_id),
        Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
    )
    .await?;
    let units_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "units",
        PropKind::Array,
        Some(*systemd_prop.id()),
        Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
    )
    .await?;
    let unit_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "unit",
        PropKind::Object,
        Some(*units_prop.id()),
        Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
    )
    .await?;
    {
        let _units_name_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "name",
            PropKind::String,
            Some(*unit_prop.id()),
            Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
        )
        .await?;
        let _units_enabled_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "enabled",
            PropKind::Boolean,
            Some(*unit_prop.id()),
            Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
        )
        .await?;
        let _units_contents_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "contents",
            PropKind::String,
            Some(*unit_prop.id()),
            Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
        )
        .await?;
    }

    // NOTE(nick): this was from "docker_image". We might need it.
    // let mut properties = HashMap::new();
    // properties.insert("image".to_owned(), serde_json::json!(""));

    // Sockets
    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    // Qualification Prototype
    let qual_func_name = "si:qualificationButaneIsValidIgnition".to_string();
    let mut qual_funcs = Func::find_by_attr(ctx, "name", &qual_func_name).await?;
    let qual_func = qual_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*schema_variant.id());
    let _ = QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Set default values after finalization.
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *variant_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["fcos"],
    )
    .await?;
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *version_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["1.4.0"],
    )
    .await?;

    // Add the ability to use docker image as an input.
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
    let (docker_image_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "docker_image",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Enable connections from the "docker_image" explicit internal provider to the
    // "/root/domain/systemd/units/" field. We need to use the appropriate function with and name
    // the argument "images".
    let units_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*units_prop.id()),
        ..base_attribute_read_context
    };
    let units_attribute_value =
        AttributeValue::find_for_context(ctx, units_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                units_attribute_value_read_context,
            ))?;
    let mut units_attribute_prototype = units_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    let transformation_func_name = "si:dockerImagesToButaneUnits".to_string();
    let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
        .await?
        .pop()
        .ok_or(FuncError::NotFoundByName(transformation_func_name))?;
    units_attribute_prototype
        .set_func_id(ctx, *transformation_func.id())
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *units_attribute_prototype.id(),
        "images",
        *docker_image_explicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}
