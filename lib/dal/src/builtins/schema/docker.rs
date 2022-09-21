use std::collections::HashMap;

use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::socket::{SocketEdgeKind, SocketKind};
use crate::{
    component::ComponentKind,
    edit_field::widget::*,
    qualification_prototype::QualificationPrototypeContext,
    schema::{SchemaVariant, UiMenu},
    socket::SocketArity,
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, BuiltinsError, BuiltinsResult, DalContext, DiagramKind, ExternalProvider,
    Func, InternalProvider, Prop, PropKind, QualificationPrototype, SchemaError, SchemaKind,
    Socket, StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    docker_hub_credential(ctx).await?;
    docker_image(ctx).await?;
    Ok(())
}

async fn docker_hub_credential(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "docker_hub_credential".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };
    schema
        .set_component_kind(ctx, ComponentKind::Credential)
        .await?;

    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(0x1e88d6)).await?;

    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;

    let mut secret_prop = Prop::new(ctx, "secret", PropKind::Integer).await?;
    secret_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;
    secret_prop
        .set_widget_kind(ctx, WidgetKind::SecretSelect)
        .await?;

    // Qualification Prototype
    let qual_func_name = "si:qualificationDockerHubLogin".to_string();
    let qual_func = Func::find_by_attr(ctx, "name", &qual_func_name)
        .await?
        .pop()
        .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*schema_variant.id());

    let _ = QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

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

    let (_output_provider, mut output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "docker_hub_credential",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;
    output_socket.set_color(ctx, Some(0x1e88d6)).await?;

    schema_variant.finalize(ctx).await?;

    // Note: I wasn't able to create a ui menu with two layers
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("credential".to_owned())).await?;
    ui_menu.set_category(ctx, Some("docker".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    Ok(())
}

async fn docker_image(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "docker_image".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(0xd61e8c)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("image")).await?;

    ui_menu.set_category(ctx, Some("docker".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let image_prop = Prop::new(ctx, "image", PropKind::String).await?;
    image_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;

    // TODO: required, validate regex: "\\d+\\/(tcp|udp)", message: "invalid exposed port entry; must be [numeric]/(tcp|udp)",
    let exposed_ports_prop = Prop::new(ctx, "ExposedPorts", PropKind::Array).await?;
    exposed_ports_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;
    let exposed_port_prop = Prop::new(ctx, "ExposedPort", PropKind::String).await?;
    exposed_port_prop
        .set_parent_prop(ctx, *exposed_ports_prop.id())
        .await?;

    // TODO: we don't have a component to have their props, but we can manually rebuild the props from what we created in this schema variant
    // This means if someone updates this function the properties will be invalid
    let mut properties = HashMap::new();
    properties.insert("image".to_owned(), serde_json::json!(""));

    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    let (_docker_hub_credential_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "docker_hub_credential",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0x1e88d6)).await?;

    let (docker_image_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "docker_image",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;
    output_socket.set_color(ctx, Some(0xd61e8c)).await?;

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
    let qual_func_name = "si:qualificationDockerImageNameInspect".to_string();
    let mut qual_funcs = Func::find_by_attr(ctx, "name", &qual_func_name).await?;
    let qual_func = qual_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*schema_variant.id());

    let _ = QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

    schema_variant.finalize(ctx).await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // Connect the "/root/si/name" field to the "/root/domain/image" field.
    let image_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*image_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await?
    .ok_or(AttributeValueError::Missing)?;
    let mut image_attribute_prototype = image_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;
    image_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
        })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *image_attribute_prototype.id(),
        "identity",
        *si_name_internal_provider.id(),
    )
    .await?;

    // Connect "/root" to the external provider.
    let root_implicit_internal_provider = InternalProvider::get_for_prop(ctx, root_prop.prop_id)
        .await?
        .ok_or(BuiltinsError::ImplicitInternalProviderNotFoundForProp(
            root_prop.prop_id,
        ))?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *docker_image_external_provider
            .attribute_prototype_id()
            .ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(
                    *docker_image_external_provider.id(),
                )
            })?,
        "identity",
        *root_implicit_internal_provider.id(),
    )
    .await?;

    let func_name = "si:dockerImageRefreshWorkflow";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
    let title = "Docker Image Resource Refresh";
    let context = WorkflowPrototypeContext {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
        ..Default::default()
    };
    WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title).await?;

    Ok(())
}
