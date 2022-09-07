use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::socket::{SocketEdgeKind, SocketKind};
use crate::{
    code_generation_prototype::CodeGenerationPrototypeContext,
    func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs,
    qualification_prototype::QualificationPrototypeContext,
    schema::{SchemaVariant, UiMenu},
    socket::SocketArity,
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, BuiltinsError,
    BuiltinsResult, CodeGenerationPrototype, CodeLanguage, DalContext, DiagramKind, Func,
    FuncError, InternalProvider, Prop, PropId, PropKind, QualificationPrototype, SchemaError,
    SchemaKind, Socket, StandardModel,
};

use crate::builtins::schema::kubernetes::doc_url;
use crate::builtins::schema::kubernetes::kubernetes_metadata::create_metadata_prop;
use crate::builtins::schema::kubernetes::kubernetes_selector::create_selector_prop;

pub async fn kubernetes_deployment(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "kubernetes_deployment".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    let (mut variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    variant.set_color(ctx, Some(0x921ed6)).await?;
    variant
        .set_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/deployment-v1/",
            )),
        )
        .await?;

    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };

    SchemaVariant::create_default_prototypes_and_values(ctx, *variant.id()).await?;

    // TODO: add validation (si-registry ensures the value is unchanged)
    let mut api_version_prop = BuiltinSchemaHelpers::create_string_prop_with_default(
        ctx,
        "apiVersion",
        "apps/v1".to_owned(),
        Some(root_prop.domain_prop_id),
        base_attribute_read_context,
    )
    .await?;
    api_version_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/deployment-v1/#Deployment",
            )),
        )
        .await?;

    // TODO: add validation (si-registry ensures the value is unchanged)
    let mut kind_prop = BuiltinSchemaHelpers::create_string_prop_with_default(
        ctx,
        "kind",
        "Deployment".to_owned(),
        Some(root_prop.domain_prop_id),
        base_attribute_read_context,
    )
    .await?;
    kind_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/deployment-v1/#Deployment",
            )),
        )
        .await?;

    let metadata_prop = create_metadata_prop(
        ctx,
        true, // is name required, note: bool is not ideal here tho
        root_prop.domain_prop_id,
    )
    .await?;

    let spec_prop = create_deployment_spec_prop(ctx, root_prop.domain_prop_id).await?;

    // Qualification Prototype
    let qualification_func_name = "si:qualificationKubevalYaml".to_owned();
    let mut qualification_funcs = Func::find_by_attr(ctx, "name", &qualification_func_name).await?;
    let qualification_func = qualification_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qualification_func_name))?;
    let mut qualification_prototype_context = QualificationPrototypeContext::new();
    qualification_prototype_context.set_schema_variant_id(*variant.id());

    let _ = QualificationPrototype::new(
        ctx,
        *qualification_func.id(),
        qualification_prototype_context,
    )
    .await?;

    // Code Generation Prototype
    let code_generation_func_name = "si:generateYAML".to_owned();
    let mut code_generation_funcs =
        Func::find_by_attr(ctx, "name", &code_generation_func_name).await?;
    let code_generation_func = code_generation_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;
    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Yaml,
        code_generation_prototype_context,
    )
    .await?;

    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    let (docker_image_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *variant.id(),
            "docker_image",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (kubernetes_namespace_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *variant.id(),
            "kubernetes_namespace",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0x85c9a3)).await?;

    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    variant.add_socket(ctx, system_socket.id()).await?;

    // TODO: abstract this boilerplate away
    let diagram_kind = schema
        .diagram_kind()
        .expect("no diagram kind for schema kind");
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("deployment".to_owned())).await?;

    ui_menu.set_category(ctx, Some("kubernetes")).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    variant.finalize(ctx).await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };

    // Connect the "domain namespace" prop to the "kubernetes_namespace" explicit internal provider.
    let domain_namespace_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *metadata_prop.id(), "namespace")
            .await?;
    let domain_namespace_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*domain_namespace_prop.id()),
        ..base_attribute_read_context
    };
    let domain_namespace_attribute_value =
        AttributeValue::find_for_context(ctx, domain_namespace_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                domain_namespace_attribute_value_read_context,
            ))?;
    let mut domain_namespace_attribute_prototype = domain_namespace_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    domain_namespace_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *domain_namespace_attribute_prototype.id(),
        "identity",
        *kubernetes_namespace_explicit_internal_provider.id(),
    )
    .await?;

    // Connect the "template namespace" prop to the "kubernetes_namespace" explicit internal provider.
    let template_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *spec_prop.id(), "template").await?;
    let template_metadata_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *template_prop.id(), "metadata").await?;
    let template_namespace_prop = BuiltinSchemaHelpers::find_child_prop_by_name(
        ctx,
        *template_metadata_prop.id(),
        "namespace",
    )
    .await?;
    let template_namespace_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*template_namespace_prop.id()),
        ..base_attribute_read_context
    };
    let template_namespace_attribute_value =
        AttributeValue::find_for_context(ctx, template_namespace_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                template_namespace_attribute_value_read_context,
            ))?;
    let mut template_namespace_attribute_prototype = template_namespace_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    template_namespace_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *template_namespace_attribute_prototype.id(),
        "identity",
        *kubernetes_namespace_explicit_internal_provider.id(),
    )
    .await?;

    // Connect the "/root/domain/spec/template/spec/containers" field to the "docker_image" explicit
    // internal provider. We need to use the appropriate function with and name the argument "images".
    let template_spec_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *template_prop.id(), "spec").await?;
    let containers_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *template_spec_prop.id(), "containers")
            .await?;
    let containers_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*containers_prop.id()),
        ..base_attribute_read_context
    };
    let containers_attribute_value =
        AttributeValue::find_for_context(ctx, containers_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                containers_attribute_value_read_context,
            ))?;
    let mut containers_attribute_prototype = containers_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    let transformation_func_name = "si:dockerImagesToK8sDeploymentContainerSpec".to_string();
    let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
        .await?
        .pop()
        .ok_or(FuncError::NotFoundByName(transformation_func_name))?;
    containers_attribute_prototype
        .set_func_id(ctx, *transformation_func.id())
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *containers_attribute_prototype.id(),
        "images",
        *docker_image_explicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}

async fn create_deployment_spec_prop(
    ctx: &DalContext,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let mut spec_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "spec",
        PropKind::Object,
        Some(parent_prop_id),
        None,
    )
    .await?;
    spec_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/deployment-v1/#DeploymentSpec",
            )),
        )
        .await?;

    let mut replicas_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "replicas",
        PropKind::Integer,
        Some(*spec_prop.id()),
        None,
    )
    .await?;
    replicas_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/deployment-v1/#DeploymentSpec",
            )),
        )
        .await?;

    let _selector_prop = create_selector_prop(ctx, *spec_prop.id()).await?;
    let _template_prop = create_pod_template_spec_prop(ctx, *spec_prop.id()).await?;

    Ok(spec_prop)
}

async fn create_pod_template_spec_prop(
    ctx: &DalContext,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let mut template_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "template",
        PropKind::Object,
        Some(parent_prop_id),
        None,
    )
    .await?;
    template_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-template-v1/#PodTemplateSpec",
            )),
        )
        .await?;

    let _metadata_prop = create_metadata_prop(
        ctx,
        true, // is name required, note: bool is not ideal here tho
        *template_prop.id(),
    )
    .await?;

    let _spec_prop = create_pod_spec_prop(ctx, *template_prop.id()).await?;

    Ok(template_prop)
}

async fn create_pod_spec_prop(ctx: &DalContext, parent_prop_id: PropId) -> BuiltinsResult<Prop> {
    let mut spec_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "spec",
        PropKind::Object,
        Some(parent_prop_id),
        None,
    )
    .await?;
    spec_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#PodSpec",
            )),
        )
        .await?;

    let mut containers_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "containers",
        PropKind::Array,
        Some(*spec_prop.id()),
        None,
    )
    .await?;
    containers_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#containers",
            )),
        )
        .await?;
    let _containers_element_prop = create_container_prop(ctx, *containers_prop.id()).await?;

    Ok(spec_prop)
}

async fn create_container_prop(ctx: &DalContext, parent_prop_id: PropId) -> BuiltinsResult<Prop> {
    let mut container_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "container",
        PropKind::Object,
        Some(parent_prop_id),
        None,
    )
    .await?;
    container_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#Container",
            )),
        )
        .await?;

    let mut name_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "name",
        PropKind::String,
        Some(*container_prop.id()),
        None,
    )
    .await?;
    name_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#Container",
            )),
        )
        .await?;

    let mut image_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "image",
        PropKind::String,
        Some(*container_prop.id()),
        None,
    )
    .await?;
    image_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#image",
            )),
        )
        .await?;

    let mut ports_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ports",
        PropKind::Array,
        Some(*container_prop.id()),
        None,
    )
    .await?;
    ports_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#ports",
            )),
        )
        .await?;
    let _ports_element_prop = create_container_port_prop(ctx, *ports_prop.id()).await?;

    Ok(container_prop)
}

async fn create_container_port_prop(
    ctx: &DalContext,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let mut port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "port",
        PropKind::Object,
        Some(parent_prop_id),
        None,
    )
    .await?;
    port_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#ports",
            )),
        )
        .await?;

    let mut container_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "containerPort",
        PropKind::Integer,
        Some(*port_prop.id()),
        None,
    )
    .await?;
    container_port_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#ports",
            )),
        )
        .await?;

    let mut protocol_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "protocol",
        PropKind::String,
        Some(*port_prop.id()),
        None,
    )
    .await?;
    protocol_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/workload-resources/pod-v1/#ports",
            )),
        )
        .await?;

    Ok(container_port_prop)
}
