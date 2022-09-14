use crate::builtins::schema::BuiltinSchemaHelpers;

use crate::socket::{SocketArity, SocketEdgeKind, SocketKind};
use crate::{
    qualification_prototype::QualificationPrototypeContext,
    schema::{SchemaVariant, UiMenu},
    AttributeContext, BuiltinsResult, DalContext, DiagramKind, Func, PropKind,
    QualificationPrototype, SchemaError, SchemaKind, Socket, StandardModel,
};

const BUTANE_DOCS_FCOS_1_4_URL: &str = "https://coreos.github.io/butane/config-fcos-v1_4/";

pub async fn migrate(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
    butane(ctx).await?;
    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`Butane`](https://coreos.github.io/butane/).
async fn butane(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
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
    {
        let _variant_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "variant",
            PropKind::String,
            Some(root_prop.domain_prop_id),
            Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
        )
        .await?;
        let _version_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "version",
            PropKind::String,
            Some(root_prop.domain_prop_id),
            Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
        )
        .await?;
    }
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

    // Wrap it up
    schema_variant.finalize(ctx).await?;

    // NOTE(nick): we will use this if we want to create intra-component connections.
    // let base_attribute_read_context = AttributeReadContext {
    //     schema_id: Some(*schema.id()),
    //     schema_variant_id: Some(*variant.id()),
    //     ..AttributeReadContext::default()
    // };

    Ok(())
}
