use std::collections::HashMap;

use crate::attribute::context::AttributeContextBuilder;
use crate::builtins::helpers::{
    find_child_prop_by_name, setup_identity_func, update_attribute_value_for_prop_and_context,
};
use crate::builtins::schema::kubernetes_metadata::create_metadata_prop;

use crate::{
    component::ComponentKind,
    edit_field::widget::*,
    func::{
        backend::{
            js_attribute::FuncBackendJsAttributeArgs,
            js_qualification::FuncBackendJsQualificationArgs,
            js_resource::FuncBackendJsResourceSyncArgs,
            validation::FuncBackendValidateStringValueArgs,
        },
        binding::FuncBinding,
        binding_return_value::FuncBindingReturnValueError,
    },
    qualification_prototype::QualificationPrototypeContext,
    resource_prototype::ResourcePrototypeContext,
    schema::{SchemaVariant, UiMenu},
    socket::{Socket, SocketArity, SocketEdgeKind},
    validation_prototype::ValidationPrototypeContext,
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, BuiltinsError, BuiltinsResult, DalContext, ExternalProvider, Func,
    FuncBackendKind, FuncBackendResponseType, FuncBindingReturnValue, InternalProvider, Prop,
    PropId, PropKind, QualificationPrototype, ResourcePrototype, Schema, SchemaError, SchemaKind,
    SchematicKind, StandardModel, ValidationPrototype,
};

mod kubernetes;
mod kubernetes_deployment;
mod kubernetes_metadata;
mod kubernetes_selector;
mod kubernetes_spec;
mod kubernetes_template;

use self::kubernetes_deployment::kubernetes_deployment;

pub async fn migrate(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    system(ctx).await?;
    application(ctx).await?;
    service(ctx).await?;
    kubernetes_service(ctx).await?;
    kubernetes_deployment(ctx).await?;
    kubernetes_namespace(ctx).await?;
    docker_hub_credential(ctx).await?;
    docker_image(ctx).await?;
    bobao(ctx).await?;

    Ok(())
}

async fn system(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "system".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concept).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    schema.set_ui_hidden(ctx, true).await?;

    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;

    let deployment_output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Deployment,
    )
    .await?;
    variant
        .add_socket(ctx, deployment_output_socket.id())
        .await?;

    let component_output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant
        .add_socket(ctx, component_output_socket.id())
        .await?;

    Ok(())
}

async fn application(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "application".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concept).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    schema.set_ui_hidden(ctx, true).await?;

    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;

    let output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Deployment,
    )
    .await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Deployment,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    Ok(())
}

async fn service(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "service".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concept).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let mut ui_menu = UiMenu::new(ctx, &(*schema.kind()).into()).await?;
    ui_menu
        .set_name(ctx, Some(schema.name().to_string()))
        .await?;

    let application_name = "application".to_string();
    ui_menu
        .set_category(ctx, Some(application_name.clone()))
        .await?;
    ui_menu
        .set_schematic_kind(ctx, SchematicKind::from(*schema.kind()))
        .await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let application_schema_results = Schema::find_by_attr(ctx, "name", &application_name).await?;
    let application_schema = application_schema_results
        .first()
        .ok_or(SchemaError::NotFoundByName(application_name))?;

    ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await?;

    let (mut variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    variant.set_color(ctx, Some(0x00b0bc)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;

    let mut input_socket = Socket::new(
        ctx,
        "service",
        &SocketEdgeKind::Configures,
        &SocketArity::Many,
        &SchematicKind::Deployment,
    )
    .await?;
    input_socket.set_color(ctx, Some(0x00b0bc)).await?;
    variant.add_socket(ctx, input_socket.id()).await?;

    let mut output_socket = Socket::new(
        ctx,
        "service",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Deployment,
    )
    .await?;
    output_socket.set_color(ctx, Some(0x00b0bc)).await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Deployment,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    Ok(())
}

async fn kubernetes_service(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "kubernetes_service".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Implementation).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (mut variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    variant
        .set_link(
            ctx,
            Some("https://kubernetes.io/docs/concepts/services-networking/service/".to_owned()),
        )
        .await?;

    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;

    let input_socket = Socket::new(
        ctx,
        "input",
        &SocketEdgeKind::Configures,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, input_socket.id()).await?;

    let output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    Ok(())
}

async fn kubernetes_namespace(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "kubernetes_namespace".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concrete).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (mut variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    variant.set_color(ctx, Some(0x85c9a3)).await?;
    variant.set_link(ctx, Some("https://v1-22.docs.kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/".to_owned())).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*variant.id());

    let mut ui_menu = UiMenu::new(ctx, &(*schema.kind()).into()).await?;
    ui_menu.set_name(ctx, Some("namespace")).await?;

    let application_name = "application".to_string();
    ui_menu
        .set_category(ctx, Some("kubernetes".to_owned()))
        .await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let application_schema_results = Schema::find_by_attr(ctx, "name", &application_name).await?;
    let application_schema = application_schema_results
        .first()
        .ok_or(SchemaError::NotFoundByName(application_name))?;
    ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await?;

    let metadata_prop = create_metadata_prop(ctx, true, root_prop.domain_prop_id).await?;

    let mut output_socket = Socket::new(
        ctx,
        "kubernetes_namespace",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    output_socket.set_color(ctx, Some(0x85c9a3)).await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *variant.id()).await?;
    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };

    // Collect the identity func information we need.
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        setup_identity_func(ctx).await?;

    // Create the "namespace" external provider and connect internally.
    // FIXME(nick): omega hack with the name.
    let external_provider = ExternalProvider::new(
        ctx,
        *schema.id(),
        *variant.id(),
        "namespace-string-output".to_string(),
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
    )
    .await?;
    let external_provider_attribute_prototype_id =
        external_provider.attribute_prototype_id().ok_or_else(|| {
            BuiltinsError::MissingAttributePrototypeForExternalProvider(*external_provider.id())
        })?;
    // update_attribute_value_for_prop_and_context(
    //     ctx,
    //     *metadata_prop.id(),
    //     Some(serde_json::json![{}]),
    //     base_attribute_read_context,
    // )
    // .await
    // .unwrap();
    let metadata_name_prop = find_child_prop_by_name(ctx, *metadata_prop.id(), "name").await?;
    // update_prop_attribute_value(
    //     ctx,
    //     *metadata_name_prop.id(),
    //     Some(serde_json::json![""]),
    //     base_attribute_read_context,
    // )
    // .await?;
    let metadata_name_implicit_internal_provider =
        InternalProvider::get_for_prop(ctx, *metadata_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*metadata_name_prop.id())
            })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *external_provider_attribute_prototype_id,
        "identity".to_string(),
        *metadata_name_implicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}

async fn docker_hub_credential(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "docker_hub_credential".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concrete).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };
    schema
        .set_component_kind(ctx, ComponentKind::Credential)
        .await?;

    let (mut variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    variant.set_color(ctx, Some(0x1e88d6)).await?;

    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
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
    let qual_args = FuncBackendJsQualificationArgs::default();
    let qual_args_json = serde_json::to_value(&qual_args)?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*variant.id());

    let mut prototype = QualificationPrototype::new(
        ctx,
        *qual_func.id(),
        qual_args_json,
        qual_prototype_context,
        "docker hub login credentials must work",
    )
    .await?;
    prototype
        .set_link(ctx, "http://hub.docker.com".into())
        .await?;

    let mut output_socket = Socket::new(
        ctx,
        "docker_hub_credential",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    output_socket.set_color(ctx, Some(0x1e88d6)).await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    let application_name = "application".to_string();

    // Note: I wasn't able to create a ui menu with two layers
    let mut ui_menu = UiMenu::new(ctx, &(*schema.kind()).into()).await?;
    ui_menu.set_name(ctx, Some("credential".to_owned())).await?;
    ui_menu.set_category(ctx, Some("docker".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let application_schema_results = Schema::find_by_attr(ctx, "name", &application_name).await?;
    let application_schema = application_schema_results
        .first()
        .ok_or(SchemaError::NotFoundByName(application_name))?;

    ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await?;

    Ok(())
}

async fn docker_image(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "docker_image".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concrete).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (mut variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    variant.set_color(ctx, Some(0xd61e8c)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*variant.id());

    let mut ui_menu = UiMenu::new(ctx, &(*schema.kind()).into()).await?;
    ui_menu.set_name(ctx, Some("image")).await?;

    let application_name = "application".to_string();
    ui_menu.set_category(ctx, Some("docker".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let application_schema_results = Schema::find_by_attr(ctx, "name", &application_name).await?;
    let application_schema = application_schema_results
        .first()
        .ok_or(SchemaError::NotFoundByName(application_name))?;
    ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await?;

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
    properties.insert("Number of Parents".to_owned(), serde_json::json!("0"));

    let mut input_socket = Socket::new(
        ctx,
        "docker_hub_credential",
        &SocketEdgeKind::Configures,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    input_socket.set_color(ctx, Some(0x1e88d6)).await?;
    variant.add_socket(ctx, input_socket.id()).await?;

    let mut output_socket = Socket::new(
        ctx,
        "docker_image",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    output_socket.set_color(ctx, Some(0xd61e8c)).await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    // Qualification Prototype
    let qual_func_name = "si:qualificationDockerImageNameInspect".to_string();
    let mut qual_funcs = Func::find_by_attr(ctx, "name", &qual_func_name).await?;
    let qual_func = qual_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
    let qual_args = FuncBackendJsQualificationArgs::default();
    let qual_args_json = serde_json::to_value(&qual_args)?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*variant.id());

    let mut prototype = QualificationPrototype::new(
        ctx,
        *qual_func.id(),
        qual_args_json,
        qual_prototype_context,
        "docker image must exist",
    )
    .await?;
    prototype.set_link(ctx, "http://docker.com".into()).await?;

    // Resource Prototype
    let resource_sync_func_name = "si:resourceSyncHammer".to_string();
    let mut resource_sync_funcs = Func::find_by_attr(ctx, "name", &resource_sync_func_name).await?;
    let resource_sync_func = resource_sync_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(resource_sync_func_name))?;
    let resource_sync_args = FuncBackendJsResourceSyncArgs::default();
    let resource_sync_args_json = serde_json::to_value(&resource_sync_args)?;
    let mut resource_sync_prototype_context = ResourcePrototypeContext::new();
    resource_sync_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = ResourcePrototype::new(
        ctx,
        *resource_sync_func.id(),
        resource_sync_args_json,
        resource_sync_prototype_context,
    )
    .await?;

    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *variant.id()).await?;
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };

    // Automatically derive the image name from the component's name. First, let's initialize the
    // field with an empty string.
    let image_context = AttributeReadContext {
        prop_id: Some(*image_prop.id()),
        ..base_attribute_read_context
    };
    dbg!("PRE IMAGE");
    let image_attribute_value = AttributeValue::find_for_context(ctx, image_context)
        .await?
        .ok_or(AttributeValueError::Missing)?;
    dbg!("POST IMAGE");
    // let domain_context = AttributeReadContext {
    //     prop_id: Some(root_prop.domain_prop_id),
    //     ..base_attribute_read_context
    // };
    // let domain_attribute_value = AttributeValue::find_for_context(ctx, domain_context)
    //     .await?
    //     .ok_or(AttributeValueError::Missing)?;
    // let image_write_context = AttributeContextBuilder::from(image_context).to_context()?;
    // let (_, updated_image_attribute_value_id, _) = AttributeValue::update_for_context(
    //     ctx,
    //     *image_attribute_value.id(),
    //     Some(*domain_attribute_value.id()),
    //     image_write_context,
    //     Some(serde_json::to_value("")?),
    //     None,
    // )
    // .await?;

    // Now, let's setup the connection with the initialized value.
    // let updated_image_attribute_value =
    //     AttributeValue::get_by_id(ctx, &updated_image_attribute_value_id)
    //         .await?
    //         .ok_or(AttributeValueError::MissingForId(
    //             updated_image_attribute_value_id,
    //         ))?;
    let mut image_attribute_prototype = image_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;
    let identity_func_name = "si:identity".to_string();
    let identity_func: Func = Func::find_by_attr(ctx, "name", &identity_func_name)
        .await?
        .pop()
        .ok_or(SchemaError::FuncNotFound(identity_func_name))?;
    image_attribute_prototype
        .set_func_id(ctx, *identity_func.id())
        .await?;
    let (_identity_func_binding, _identity_func_binding_return_value) =
        FuncBinding::find_or_create_and_execute(
            ctx,
            serde_json::json![{ "identity": null }],
            *identity_func.id(),
        )
        .await?;

    // Finally, find the implicit internal provider for the component's name and create the
    // attribute argument prototype.
    let si_name_prop = find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
        })?;
    let _argument = AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *image_attribute_prototype.id(),
        "identity".to_string(),
        *si_name_internal_provider.id(),
    )
    .await?;

    Ok(())
}

async fn bobao(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    let name = "bobão".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concrete).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*variant.id());

    let mut ui_menu = UiMenu::new(ctx, &(*schema.kind()).into()).await?;
    ui_menu.set_name(ctx, Some("bobão")).await?;

    let application_name = "application".to_string();
    ui_menu.set_category(ctx, Some("docker".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let application_schema_results = Schema::find_by_attr(ctx, "name", &application_name).await?;
    let application_schema = application_schema_results
        .first()
        .ok_or(SchemaError::NotFoundByName(application_name))?;
    ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await?;

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let mut validation_prototype_ctx = ValidationPrototypeContext::default();
    validation_prototype_ctx.set_schema_id(*schema.id());
    validation_prototype_ctx.set_schema_variant_id(*variant.id());

    let text_prop = Prop::new(ctx, "text", PropKind::String).await?;
    text_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;
    validation_prototype_ctx.set_prop_id(*text_prop.id());
    let mut prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "Tupi or not Tupi, that is the question".to_owned(), // https://en.wikipedia.org/wiki/Manifesto_Antrop%C3%B3fago
        ))?,
        validation_prototype_ctx.clone(),
    )
    .await?;
    prototype
        .set_link(
            ctx,
            Some("https://en.wikipedia.org/wiki/Manifesto_Antrop%C3%B3fago".to_owned()),
        )
        .await?;

    let integer_prop = Prop::new(ctx, "integer", PropKind::Integer).await?;
    integer_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;

    let boolean_prop = Prop::new(ctx, "boolean", PropKind::Boolean).await?;
    boolean_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;

    let object_prop = Prop::new(ctx, "object", PropKind::Object).await?;
    object_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;
    validation_prototype_ctx.set_prop_id(*integer_prop.id());
    let mut prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "My office is at the beach".to_owned(),
        ))?,
        validation_prototype_ctx.clone(),
    )
    .await?;
    prototype
        .set_link(
            ctx,
            Some("https://www.youtube.com/watch?v=JiVsAnIgBIs".to_owned()),
        )
        .await?;

    let child_prop = Prop::new(ctx, "child", PropKind::String).await?;
    child_prop.set_parent_prop(ctx, *object_prop.id()).await?;

    let map_prop = Prop::new(ctx, "map", PropKind::Object).await?;
    map_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;
    validation_prototype_ctx.set_prop_id(*map_prop.id());
    let mut prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "I'm just a latin american guy\nWith no money in the bank, no important relatives\nComing from the country".to_owned(),
        ))?,
        validation_prototype_ctx.clone(),
    )
        .await?;
    prototype
        .set_link(
            ctx,
            Some("https://www.youtube.com/watch?v=8VcZURSMetg".to_owned()),
        )
        .await?;

    let array_prop = Prop::new(ctx, "array", PropKind::Object).await?;
    array_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;
    validation_prototype_ctx.set_prop_id(*array_prop.id());
    let mut prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "I'm brazilian, of median stature\nI like so-and-so but the other one is who wants me"
                .to_owned(),
        ))?,
        validation_prototype_ctx,
    )
    .await?;
    prototype
        .set_link(
            ctx,
            Some("https://www.youtube.com/watch?v=Vz73zZriafQ".to_owned()),
        )
        .await?;

    let mut secret_prop = Prop::new(ctx, "secret", PropKind::Integer).await?;
    secret_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await?;
    secret_prop
        .set_widget_kind(ctx, WidgetKind::SecretSelect)
        .await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    Ok(())
}

async fn create_schema(
    ctx: &DalContext<'_, '_>,
    schema_name: &str,
    schema_kind: &SchemaKind,
) -> BuiltinsResult<Option<Schema>> {
    // TODO(nick): there's one issue here. If the schema kind has changed, then this check will be
    // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
    // This should be fine since this code should likely only last as long as default schemas need to
    // be created... which is hopefully not long.... hopefully...
    let default_schema_exists = !Schema::find_by_attr(ctx, "name", &schema_name.to_string())
        .await?
        .is_empty();

    // TODO(nick): this should probably return an "AlreadyExists" error instead of "None", but
    // since the calling function would have to deal with the result similarly, this should suffice
    // for now.
    match default_schema_exists {
        true => Ok(None),
        false => {
            let schema =
                Schema::new(ctx, schema_name, schema_kind, &ComponentKind::Standard).await?;
            Ok(Some(schema))
        }
    }
}

/// Creates a [`Prop`]. While a base [`AttributeReadContext`] is required for this function, it is
/// only used when a parent [`PropId`] is provided.
#[allow(clippy::too_many_arguments)]
pub async fn create_prop(
    ctx: &DalContext<'_, '_>,
    prop_name: &str,
    prop_kind: PropKind,
    parent_prop_id: Option<PropId>,
) -> BuiltinsResult<Prop> {
    let prop = Prop::new(ctx, prop_name, prop_kind).await?;
    if let Some(parent_prop_id) = parent_prop_id {
        prop.set_parent_prop(ctx, parent_prop_id).await?;
    }
    Ok(prop)
}

pub async fn create_string_prop_with_default(
    ctx: &DalContext<'_, '_>,
    prop_name: &str,
    default_string: String,
    parent_prop_id: Option<PropId>,
    _base_attribute_read_context: AttributeReadContext,
) -> BuiltinsResult<Prop> {
    let prop = create_prop(ctx, prop_name, PropKind::String, parent_prop_id).await?;

    let mut func = Func::new(
        ctx,
        &format!("si:setDefaultToProp{:?}", prop.id()),
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");
    func.set_handler(ctx, Some("defaultValue")).await?;
    func.set_code_base64(
        ctx,
        Some(base64::encode(&format!(
            "function defaultValue(component) {{ return \"{default_string}\"; }}"
        ))),
    )
    .await?;

    let (func_binding, created) = FuncBinding::find_or_create(
        ctx,
        // The default run doesn't have useful information, but it's just a reference for future reruns
        serde_json::to_value(FuncBackendJsAttributeArgs {
            component: veritech::ResolverFunctionComponent {
                data: veritech::ComponentView {
                    properties: serde_json::json!({}),
                    system: None,
                    kind: veritech::ComponentKind::Standard,
                },
                parents: vec![],
            },
        })?,
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    if created {
        func_binding.execute(ctx).await?;
    }

    let func_binding_return_value =
        FuncBindingReturnValue::get_by_func_binding_id(ctx, *func_binding.id())
            .await?
            .ok_or(FuncBindingReturnValueError::Missing)?;

    let attribute_value_context = AttributeReadContext {
        prop_id: Some(*prop.id()),
        ..AttributeReadContext::default()
    };

    dbg!("PRE PROP");
    let mut attribute_value = AttributeValue::find_for_context(ctx, attribute_value_context)
        .await?
        .ok_or(AttributeValueError::Missing)?;
    dbg!("POST PROP");
    attribute_value
        .set_func_binding_id(ctx, *func_binding.id())
        .await?;
    attribute_value
        .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
        .await?;

    let mut attribute_prototype = attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;
    attribute_prototype.set_func_id(ctx, *func.id()).await?;

    Ok(prop)
}
