use std::collections::HashMap;

use crate::func::backend::js_attribute::FuncBackendJsAttributeArgs;
use crate::func::backend::js_qualification::FuncBackendJsQualificationArgs;
use crate::func::backend::js_resource::FuncBackendJsResourceSyncArgs;
use crate::func::backend::validation::FuncBackendValidateStringValueArgs;
use crate::qualification_prototype::QualificationPrototypeContext;
use crate::resource_prototype::ResourcePrototypeContext;
use crate::schema::{SchemaResult, SchemaVariant, UiMenu};
use crate::socket::input::InputSocket;
use crate::socket::{Socket, SocketArity, SocketEdgeKind};
use crate::{
    component::ComponentKind, edit_field::widget::*, func::binding::FuncBinding,
    validation_prototype::ValidationPrototypeContext, Func, FuncBackendKind,
    FuncBackendResponseType, Prop, PropId, PropKind, QualificationPrototype, ResourcePrototype,
    Schema, SchemaError, SchemaKind, SchemaVariantId, SchematicKind, StandardModel,
    ValidationPrototype,
};
use crate::{
    AttributeContext, AttributePrototype, AttributePrototypeError, AttributeReadContext,
    AttributeValue, AttributeValueError, DalContext, SchemaId,
};
use telemetry::prelude::*;

mod kubernetes_deployment;
mod kubernetes_metadata;
mod kubernetes_selector;
mod kubernetes_spec;
mod kubernetes_template;

use self::kubernetes_deployment::kubernetes_deployment;

pub async fn migrate(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
    system(ctx).await?;
    application(ctx).await?;
    service(ctx).await?;
    kubernetes_service(ctx).await?;
    kubernetes_deployment(ctx).await?;
    docker_hub_credential(ctx).await?;
    docker_image(ctx).await?;

    Ok(())
}

async fn system(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
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

async fn application(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
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

async fn service(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
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

    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
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

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

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

    let output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Deployment,
    )
    .await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    Ok(())
}

async fn kubernetes_service(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
    let name = "kubernetes_service".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Implementation).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
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

async fn docker_hub_credential(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
    let name = "docker_hub_credential".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concrete).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };
    schema
        .set_component_kind(ctx, ComponentKind::Credential)
        .await?;

    let (variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };

    let mut secret_prop = Prop::new(ctx, "secret", PropKind::Integer).await?;
    secret_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id, base_attribute_read_context)
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

    // Note: This is not right; each schema needs its own socket types.
    //       Also, they should clearly inherit from the core schema? Something
    //       for later.
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

async fn docker_image(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
    let name = "docker_image".to_string();
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
    let root_prop_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root_prop.domain_prop_id),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*variant.id()),
            ..AttributeReadContext::default()
        },
    )
    .await?
    .pop()
    .ok_or(AttributeValueError::Missing)?;

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

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };

    let image_prop = Prop::new(ctx, "image", PropKind::String).await?;
    image_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id, base_attribute_read_context)
        .await?;

    // TODO: required, validate regex: "\\d+\\/(tcp|udp)", message: "invalid exposed port entry; must be [numeric]/(tcp|udp)",
    let exposed_ports_prop = Prop::new(
        ctx,
        "ExposedPorts",
        PropKind::Array, // Note: we should have a way to specify that this is an array of Integer
    )
    .await?;
    exposed_ports_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id, base_attribute_read_context)
        .await?;

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let mut validation_prototype_ctx = ValidationPrototypeContext::default();
    validation_prototype_ctx.set_prop_id(*image_prop.id());
    validation_prototype_ctx.set_schema_id(*schema.id());
    validation_prototype_ctx.set_schema_variant_id(*variant.id());
    ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "gambiarra".to_owned(),
        ))?,
        validation_prototype_ctx.clone(),
    )
    .await?;

    ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "Tupi or not Tupi, that is the question".to_owned(), // https://en.wikipedia.org/wiki/Manifesto_Antrop%C3%B3fago
        ))?,
        validation_prototype_ctx,
    )
    .await?;

    let number_of_parents_prop = Prop::new(
        ctx,
        "Number of Parents",
        PropKind::String, // Should be integer, but the js integer backend isn't 100% there yet
    )
    .await?;
    number_of_parents_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id, base_attribute_read_context)
        .await?;

    // TODO: we don't have a component to have their props, but we can manually rebuild the props from what we created in this schema variant
    // This means if someone updates this function the properties will be invalid
    let mut properties = HashMap::new();
    properties.insert("image".to_owned(), serde_json::json!(""));
    properties.insert("Number of Parents".to_owned(), serde_json::json!("0"));

    let func_name = "si:numberOfParents".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let (func_binding, _) = FuncBinding::find_or_create(
        ctx,
        serde_json::to_value(FuncBackendJsAttributeArgs {
            component: veritech::ResolverFunctionComponent {
                data: veritech::ComponentView {
                    properties: serde_json::to_value(properties)?,
                    kind: veritech::ComponentKind::Standard,
                    system: None,
                },
                parents: vec![],
            },
        })?,
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    let func_binding_return_value = func_binding.execute(ctx).await?;

    let number_of_parents_prop_context = attribute_context_builder
        .clone()
        .set_prop_id(*number_of_parents_prop.id())
        .to_context()?;
    let attribute_prototype =
        AttributePrototype::list_for_context(ctx, number_of_parents_prop_context)
            .await?
            .pop()
            .ok_or(AttributePrototypeError::Missing)?;
    AttributePrototype::update_for_context(
        ctx,
        *attribute_prototype.id(),
        number_of_parents_prop_context,
        *func.id(),
        *func_binding.id(),
        *func_binding_return_value.id(),
        Some(*root_prop_attribute_value.id()),
        Some(
            *AttributeValue::find_for_context(ctx, number_of_parents_prop_context.into())
                .await?
                .pop()
                .ok_or(AttributeValueError::Missing)?
                .id(),
        ),
    )
    .await?;

    // Note: This is not right; each schema needs its own socket types.
    //       Also, they should clearly inherit from the core schema? Something
    //       for later.
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
        "docker image name must match the component name",
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

    // FIXME(nick): remove this dummy socket once it is unneeded.
    let _dummy_input_socket = InputSocket::new(
        ctx,
        *image_prop.id(),
        *schema.id(),
        *variant.id(),
        Some("image".to_string()),
        true,
        None,
    );

    Ok(())
}

async fn create_schema(
    ctx: &DalContext<'_, '_>,
    schema_name: &str,
    schema_kind: &SchemaKind,
) -> SchemaResult<Option<Schema>> {
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
    base_attribute_read_context: AttributeReadContext,
) -> SchemaResult<Prop> {
    let prop = Prop::new(ctx, prop_name, prop_kind).await?;
    if let Some(parent_prop_id) = parent_prop_id {
        prop.set_parent_prop(ctx, parent_prop_id, base_attribute_read_context)
            .await?;
    }
    Ok(prop)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_string_prop_with_default(
    ctx: &DalContext<'_, '_>,
    prop_name: &str,
    default_string: String,
    parent_prop_id: Option<PropId>,
    base_attribute_read_context: AttributeReadContext,
) -> SchemaResult<Prop> {
    let prop = create_prop(
        ctx,
        prop_name,
        PropKind::String,
        parent_prop_id,
        base_attribute_read_context,
    )
    .await?;

    let mut func = Func::new(
        ctx,
        &format!("si:setDefaultToProp{}", prop.id()),
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

    // TODO: Set up AttribuePrototype & AttributeValue appropriately

    Ok(prop)
}

pub struct RootProp {
    pub si_prop_id: PropId,
    pub domain_prop_id: PropId,
}

/// Returns si-specific prop id and domain-specific prop id, respectfully
#[instrument(skip_all)]
#[allow(clippy::too_many_arguments)]
pub async fn create_root_prop(
    ctx: &DalContext<'_, '_>,
    schema_id: SchemaId,
    variant_id: SchemaVariantId,
) -> SchemaResult<RootProp> {
    let mut variant_context_builder = AttributeContext::builder();
    variant_context_builder
        .set_schema_id(schema_id)
        .set_schema_variant_id(variant_id);

    let func_name = "si:setPropObject".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;

    let (func_binding, created) = FuncBinding::find_or_create(
        ctx,
        // Shortcut creating a FuncBackendPropObjectArgs
        serde_json::json!({ "value": {} }),
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    if created {
        func_binding.execute(ctx).await?;
    }

    let root_prop = Prop::new(ctx, "root", PropKind::Object).await?;
    root_prop.add_schema_variant(ctx, &variant_id).await?;

    let root_context = variant_context_builder
        .clone()
        .set_prop_id(*root_prop.id())
        .to_context()?;
    let (_, root_value_id) = AttributeValue::update_for_context(
        ctx,
        *AttributeValue::find_for_context(ctx, root_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        None,
        root_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(schema_id),
        schema_variant_id: Some(variant_id),
        ..AttributeReadContext::default()
    };

    let si_specific_prop = Prop::new(ctx, "si", PropKind::Object).await?;
    si_specific_prop
        .set_parent_prop(ctx, *root_prop.id(), base_attribute_read_context)
        .await?;

    let si_context = variant_context_builder
        .clone()
        .set_prop_id(*si_specific_prop.id())
        .to_context()?;
    let (_, si_value_id) = AttributeValue::update_for_context(
        ctx,
        *AttributeValue::find_for_context(ctx, si_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        Some(root_value_id),
        si_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await?;

    let si_name_prop = Prop::new(ctx, "name", PropKind::String).await?;
    si_name_prop
        .set_parent_prop(ctx, *si_specific_prop.id(), base_attribute_read_context)
        .await?;

    let si_name_context = variant_context_builder
        .clone()
        .set_prop_id(*si_name_prop.id())
        .to_context()?;
    AttributeValue::update_for_context(
        ctx,
        *AttributeValue::find_for_context(ctx, si_name_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        Some(si_value_id),
        si_name_context,
        None,
        None,
    )
    .await?;

    let domain_specific_prop = Prop::new(ctx, "domain", PropKind::Object).await?;
    domain_specific_prop
        .set_parent_prop(ctx, *root_prop.id(), base_attribute_read_context)
        .await?;

    let domain_context = variant_context_builder
        .clone()
        .set_prop_id(*domain_specific_prop.id())
        .to_context()?;
    AttributeValue::update_for_context(
        ctx,
        *AttributeValue::find_for_context(ctx, domain_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        Some(root_value_id),
        domain_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await?;

    Ok(RootProp {
        si_prop_id: *si_specific_prop.id(),
        domain_prop_id: *domain_specific_prop.id(),
    })
}
