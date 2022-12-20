use crate::schema::variant::leaves::LeafKind;
use crate::{builtins::schema::MigrationDriver, schema::variant::leaves::LeafInputLocation};
use crate::{component::ComponentKind, schema::variant::leaves::LeafInput};
use crate::{
    func::argument::FuncArgument, schema::SchemaUiMenu, socket::SocketArity,
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, AttributeValueError,
    BuiltinsError, BuiltinsResult, DalContext, DiagramKind, ExternalProvider, InternalProvider,
    PropKind, SchemaError, SchemaKind, SchemaVariant, StandardModel,
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
pub fn doc_url(path: impl AsRef<str>) -> String {
    format!(
        "https://v{}.docs.kubernetes.io/docs/{}",
        DEFAULT_KUBERNETES_API_VERSION.replace('.', "-"),
        path.as_ref(),
    )
}

impl MigrationDriver {
    pub async fn migrate_kubernetes(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_kubernetes_namespace(ctx).await?;
        self.migrate_kubernetes_deployment(ctx).await?;
        Ok(())
    }

    async fn migrate_kubernetes_namespace(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _) = match self
            .create_schema_and_variant(
                ctx,
                "Kubernetes Namespace",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(KUBERNETES_NODE_COLOR),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        schema_variant.set_link(ctx, Some("https://v1-22.docs.kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/".to_owned())).await?;

        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
        let ui_menu = SchemaUiMenu::new(ctx, "Namespace", "Kubernetes", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        let metadata_prop = self
            .create_kubernetes_metadata_prop(ctx, true, root_prop.domain_prop_id)
            .await?;

        // Add code generation
        let code_generation_func_id = self.get_func_id("si:generateYAML").ok_or(
            BuiltinsError::FuncNotFoundInMigrationCache("si:generateYAML"),
        )?;
        let code_generation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "domain", code_generation_func_id)
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        "si:generateYAML".to_string(),
                        "domain".to_string(),
                    )
                })?;
        SchemaVariant::add_leaf(
            ctx,
            code_generation_func_id,
            *schema_variant.id(),
            LeafKind::CodeGeneration,
            vec![LeafInput {
                location: LeafInputLocation::Domain,
                arg_id: *code_generation_func_argument.id(),
            }],
        )
        .await?;

        // Create sockets
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

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

        self.finalize_schema_variant(ctx, &schema_variant, &root_prop)
            .await?;

        // Connect the "/root/si/name" field to the "/root/domain/metadata/name" field.
        let metadata_name_prop = self
            .find_child_prop_by_name(ctx, *metadata_prop.id(), "name")
            .await?;
        let metadata_name_prop_id = *metadata_name_prop.id();
        let metadata_name_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext::default_with_prop(metadata_name_prop_id),
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
        let si_name_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "name")
            .await?;
        let si_name_internal_provider = InternalProvider::find_for_prop(ctx, *si_name_prop.id())
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
        let metadata_name_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, metadata_name_prop_id)
                .await?
                .ok_or(BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                    metadata_name_prop_id,
                ))?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *external_provider_attribute_prototype_id,
            identity_func_item.func_argument_id,
            *metadata_name_implicit_internal_provider.id(),
        )
        .await?;

        Ok(())
    }

    async fn migrate_kubernetes_deployment(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _) = match self
            .create_schema_and_variant(
                ctx,
                "Kubernetes Deployment",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(KUBERNETES_NODE_COLOR),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        schema_variant
            .set_link(
                ctx,
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/deployment-v1/",
                )),
            )
            .await?;

        let api_version_prop = self
            .create_prop(
                ctx,
                "apiVersion",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/deployment-v1/#Deployment",
                )),
            )
            .await?;
        let kind_prop = self
            .create_prop(
                ctx,
                "kind",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/deployment-v1/#Deployment",
                )),
            )
            .await?;

        let metadata_prop = self
            .create_kubernetes_metadata_prop(
                ctx,
                true, // is name required, note: bool is not ideal here tho
                root_prop.domain_prop_id,
            )
            .await?;

        let spec_prop = self
            .create_kubernetes_deployment_spec_prop(ctx, root_prop.domain_prop_id)
            .await?;

        // Qualifications
        let (qualification_func_id, qualification_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:qualificationKubevalYaml", "code")
            .await?;
        SchemaVariant::add_leaf(
            ctx,
            qualification_func_id,
            *schema_variant.id(),
            LeafKind::Qualification,
            vec![LeafInput {
                location: LeafInputLocation::Code,
                arg_id: qualification_func_argument_id,
            }],
        )
        .await?;

        // Add code generation
        let code_generation_func_id = self.get_func_id("si:generateYAML").ok_or(
            BuiltinsError::FuncNotFoundInMigrationCache("si:generateYAML"),
        )?;
        let code_generation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "domain", code_generation_func_id)
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        "si:generateYAML".to_string(),
                        "domain".to_string(),
                    )
                })?;
        SchemaVariant::add_leaf(
            ctx,
            code_generation_func_id,
            *schema_variant.id(),
            LeafKind::CodeGeneration,
            vec![LeafInput {
                location: LeafInputLocation::Domain,
                arg_id: *code_generation_func_argument.id(),
            }],
        )
        .await?;

        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (docker_image_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
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

        let diagram_kind = schema
            .diagram_kind()
            .expect("no diagram kind for schema kind");
        let ui_menu = SchemaUiMenu::new(ctx, "Deployment", "Kubernetes", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        self.finalize_schema_variant(ctx, &schema_variant, &root_prop)
            .await?;

        // Set default values after finalization.
        self.set_default_value_for_prop(ctx, *api_version_prop.id(), serde_json::json!["apps/v1"])
            .await?;
        self.set_default_value_for_prop(ctx, *kind_prop.id(), serde_json::json!["Deployment"])
            .await?;

        // Connect the "domain namespace" prop to the "kubernetes_namespace" explicit internal provider.
        let domain_namespace_prop = self
            .find_child_prop_by_name(ctx, *metadata_prop.id(), "namespace")
            .await?;
        let domain_namespace_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*domain_namespace_prop.id());
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
        let template_prop = self
            .find_child_prop_by_name(ctx, *spec_prop.id(), "template")
            .await?;
        let template_metadata_prop = self
            .find_child_prop_by_name(ctx, *template_prop.id(), "metadata")
            .await?;
        let template_namespace_prop = self
            .find_child_prop_by_name(ctx, *template_metadata_prop.id(), "namespace")
            .await?;
        let template_namespace_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*template_namespace_prop.id());
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
        let template_spec_prop = self
            .find_child_prop_by_name(ctx, *template_prop.id(), "spec")
            .await?;
        let containers_prop = self
            .find_child_prop_by_name(ctx, *template_spec_prop.id(), "containers")
            .await?;
        let containers_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*containers_prop.id());
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
        let (transformation_func_id, transformation_func_argument_id) = self
            .find_func_and_single_argument_by_names(
                ctx,
                "si:dockerImagesToK8sDeploymentContainerSpec",
                "images",
            )
            .await?;
        containers_attribute_prototype
            .set_func_id(ctx, transformation_func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *containers_attribute_prototype.id(),
            transformation_func_argument_id,
            *docker_image_explicit_internal_provider.id(),
        )
        .await?;

        Ok(())
    }
}
