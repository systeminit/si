use crate::builtins::schema::{BuiltinSchemaDriver, BuiltinSchemaHelpers};
use crate::component::ComponentKind;
use crate::prototype_context::PrototypeContext;
use crate::qualification_prototype::QualificationPrototypeContext;
use crate::socket::{SocketEdgeKind, SocketKind};
use crate::{
    code_generation_prototype::CodeGenerationPrototypeContext, func::argument::FuncArgument,
    func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs, schema::SchemaUiMenu,
    socket::SocketArity, AttributeContext, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, AttributeValueError, BuiltinsError, BuiltinsResult, CodeGenerationPrototype,
    CodeLanguage, DalContext, DiagramKind, ExternalProvider, Func, FuncError, InternalProvider,
    PropKind, QualificationPrototype, SchemaError, SchemaKind, Socket, StandardModel,
    WorkflowPrototype, WorkflowPrototypeContext,
};

mod kubernetes_deployment_spec;
mod kubernetes_metadata;
mod kubernetes_selector;
mod kubernetes_spec;
mod kubernetes_template;

// This node color is purely meant the complement existing node colors. It does not reflect an
// official branding Kubernetes color.
const KUBERNETES_NODE_COLOR: i64 = 0x30BA78;

/// The default Kubernetes API version used when creating documentation URLs.
const DEFAULT_KUBERNETES_API_VERSION: &str = "1.22";

/// Provides the documentation URL prefix for a given Kubernetes documentation URL path.
pub fn kubernetes_doc_url(path: impl AsRef<str>) -> String {
    format!(
        "https://v{}.docs.kubernetes.io/docs/{}",
        DEFAULT_KUBERNETES_API_VERSION.replace('.', "-"),
        path.as_ref(),
    )
}

impl BuiltinSchemaDriver {
    pub async fn migrate_kubernetes(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_kubernetes_namespace(ctx).await?;
        self.migrate_kubernetes_deployment(ctx).await?;
        Ok(())
    }

    async fn migrate_kubernetes_namespace(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop) =
            match BuiltinSchemaHelpers::create_schema_and_variant(
                ctx,
                "Kubernetes Namespace",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(KUBERNETES_NODE_COLOR),
            )
            .await?
            {
                Some(tuple) => tuple,
                None => return Ok(()),
            };

        schema_variant.set_link(ctx, Some("https://v1-22.docs.kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/".to_owned())).await?;

        let mut attribute_context_builder = AttributeContext::builder();
        attribute_context_builder
            .set_schema_id(*schema.id())
            .set_schema_variant_id(*schema_variant.id());

        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
        let ui_menu = SchemaUiMenu::new(ctx, "Namespace", "Kubernetes", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        let metadata_prop =
            kubernetes_metadata::create_metadata_prop(ctx, true, root_prop.domain_prop_id).await?;

        // Code Generation Prototype
        let code_generation_func_id = self
            .get_func_id("si:generateYAML")
            .ok_or(BuiltinsError::FuncNotFoundInCache("si:generateYAML"))?;
        let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
        let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
        let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
        code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

        let _prototype = CodeGenerationPrototype::new(
            ctx,
            code_generation_func_id,
            code_generation_args_json,
            CodeLanguage::Yaml,
            code_generation_prototype_context,
        )
        .await?;

        // Create sockets
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInCache("si:identity"))?;

        let (external_provider, mut output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Kubernetes Namespace",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
        output_socket.set_color(ctx, Some(0x85c9a3)).await?;

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

        schema_variant.finalize(ctx).await?;

        // Connect the "/root/si/name" field to the "/root/domain/metadata/name" field.
        let base_attribute_read_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            ..AttributeReadContext::default()
        };
        let metadata_name_prop =
            BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *metadata_prop.id(), "name").await?;
        let metadata_name_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*metadata_name_prop.id()),
                ..base_attribute_read_context
            },
        )
        .await?
        .ok_or(AttributeValueError::Missing)?;
        let mut metadata_name_attribute_prototype = metadata_name_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        metadata_name_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        let si_name_prop =
            BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name")
                .await?;
        let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
            })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *metadata_name_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *si_name_internal_provider.id(),
        )
        .await?;

        // Connect the "/root/domain/metadata/name" prop to the external provider.
        let external_provider_attribute_prototype_id =
            external_provider.attribute_prototype_id().ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(*external_provider.id())
            })?;
        let metadata_name_prop =
            BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *metadata_prop.id(), "name").await?;
        let metadata_name_implicit_internal_provider =
            InternalProvider::get_for_prop(ctx, *metadata_name_prop.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::ImplicitInternalProviderNotFoundForProp(*metadata_name_prop.id())
                })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *external_provider_attribute_prototype_id,
            identity_func_item.func_argument_id,
            *metadata_name_implicit_internal_provider.id(),
        )
        .await?;

        let mut context = WorkflowPrototypeContext::new(); // workspace level
        context.schema_id = *schema.id();
        context.schema_variant_id = *schema_variant.id();
        let title = "What Is Love";
        let func_name = "si:whatIsLoveWorkflow";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title).await?;

        Ok(())
    }

    async fn migrate_kubernetes_deployment(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop) =
            match BuiltinSchemaHelpers::create_schema_and_variant(
                ctx,
                "Kubernetes Deployment",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(KUBERNETES_NODE_COLOR),
            )
            .await?
            {
                Some(tuple) => tuple,
                None => return Ok(()),
            };

        schema_variant
            .set_link(
                ctx,
                Some(kubernetes_doc_url(
                    "reference/kubernetes-api/workload-resources/deployment-v1/",
                )),
            )
            .await?;

        let api_version_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "apiVersion",
            PropKind::String,
            None,
            Some(root_prop.domain_prop_id),
            Some(kubernetes_doc_url(
                "reference/kubernetes-api/workload-resources/deployment-v1/#Deployment",
            )),
        )
        .await?;
        let kind_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "kind",
            PropKind::String,
            None,
            Some(root_prop.domain_prop_id),
            Some(kubernetes_doc_url(
                "reference/kubernetes-api/workload-resources/deployment-v1/#Deployment",
            )),
        )
        .await?;

        let metadata_prop = kubernetes_metadata::create_metadata_prop(
            ctx,
            true, // is name required, note: bool is not ideal here tho
            root_prop.domain_prop_id,
        )
        .await?;

        let spec_prop =
            kubernetes_deployment_spec::create_deployment_spec_prop(ctx, root_prop.domain_prop_id)
                .await?;

        // Qualification Prototype
        let qualification_func_name = "si:qualificationKubevalYaml".to_owned();
        let mut qualification_funcs =
            Func::find_by_attr(ctx, "name", &qualification_func_name).await?;
        let qualification_func = qualification_funcs
            .pop()
            .ok_or(SchemaError::FuncNotFound(qualification_func_name))?;
        let mut qualification_prototype_context = QualificationPrototypeContext::new();
        qualification_prototype_context.set_schema_variant_id(*schema_variant.id());

        let _ = QualificationPrototype::new(
            ctx,
            *qualification_func.id(),
            qualification_prototype_context,
        )
        .await?;

        // Code Generation Prototype
        let code_generation_func_id = self
            .get_func_id("si:generateYAML")
            .ok_or(BuiltinsError::FuncNotFoundInCache("si:generateYAML"))?;
        let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
        let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
        let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
        code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

        let _prototype = CodeGenerationPrototype::new(
            ctx,
            code_generation_func_id,
            code_generation_args_json,
            CodeLanguage::Yaml,
            code_generation_prototype_context,
        )
        .await?;

        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInCache("si:identity"))?;

        let (docker_image_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema.id(),
                *schema_variant.id(),
                "Container Image",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        input_socket.set_color(ctx, Some(0xd61e8c)).await?;

        let (kubernetes_namespace_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema.id(),
                *schema_variant.id(),
                "Kubernetes Namespace",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
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
        schema_variant.add_socket(ctx, system_socket.id()).await?;

        let diagram_kind = schema
            .diagram_kind()
            .expect("no diagram kind for schema kind");
        let ui_menu = SchemaUiMenu::new(ctx, "Deployment", "Kubernetes", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        schema_variant.finalize(ctx).await?;

        // Set default values after finalization.
        BuiltinSchemaHelpers::set_default_value_for_prop(
            ctx,
            *api_version_prop.id(),
            *schema.id(),
            *schema_variant.id(),
            serde_json::json!["apps/v1"],
        )
        .await?;
        BuiltinSchemaHelpers::set_default_value_for_prop(
            ctx,
            *kind_prop.id(),
            *schema.id(),
            *schema_variant.id(),
            serde_json::json!["Deployment"],
        )
        .await?;

        let base_attribute_read_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
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
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *domain_namespace_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *kubernetes_namespace_explicit_internal_provider.id(),
        )
        .await?;

        // Connect the "template namespace" prop to the "kubernetes_namespace" explicit internal provider.
        let template_prop =
            BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *spec_prop.id(), "template").await?;
        let template_metadata_prop =
            BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *template_prop.id(), "metadata")
                .await?;
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
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *template_namespace_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *kubernetes_namespace_explicit_internal_provider.id(),
        )
        .await?;

        // Connect the "/root/domain/spec/template/spec/containers" field to the "Container Image" explicit
        // internal provider. We need to use the appropriate function with and name the argument "images".
        let template_spec_prop =
            BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *template_prop.id(), "spec").await?;
        let containers_prop = BuiltinSchemaHelpers::find_child_prop_by_name(
            ctx,
            *template_spec_prop.id(),
            "containers",
        )
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
            .ok_or_else(|| FuncError::NotFoundByName(transformation_func_name.clone()))?;
        let images_arg =
            FuncArgument::find_by_name_for_func(ctx, "images", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        *transformation_func.id(),
                        "images".to_string(),
                    )
                })?;
        containers_attribute_prototype
            .set_func_id(ctx, *transformation_func.id())
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *containers_attribute_prototype.id(),
            *images_arg.id(),
            *docker_image_explicit_internal_provider.id(),
        )
        .await?;

        Ok(())
    }
}
