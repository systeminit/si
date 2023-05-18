use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafKind;
use crate::{
    builtins::schema::MigrationDriver, schema::variant::leaves::LeafInputLocation, ComponentType,
    Prop, PropId, SchemaVariantId,
};
use crate::{component::ComponentKind, schema::variant::leaves::LeafInput};
use crate::{
    func::argument::FuncArgument, socket::SocketArity, AttributePrototypeArgument,
    AttributeReadContext, AttributeValue, AttributeValueError, BuiltinsError, BuiltinsResult,
    DalContext, ExternalProvider, InternalProvider, PropKind, SchemaVariant, StandardModel,
};

/// The default Kubernetes API version used when creating documentation URLs.
const DEFAULT_KUBERNETES_API_VERSION: &str = "1.22";

/// Provides the documentation URL prefix for a given Kubernetes documentation URL path.
fn doc_url(path: impl AsRef<str>) -> String {
    format!(
        "https://v{}.docs.kubernetes.io/docs/{}",
        DEFAULT_KUBERNETES_API_VERSION.replace('.', "-"),
        path.as_ref(),
    )
}

impl MigrationDriver {
    pub async fn migrate_kubernetes_namespace(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let (mut schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    "Kubernetes Namespace",
                    Some("Namespace"),
                    ui_menu_category,
                    node_color,
                    ComponentKind::Standard,
                    None,
                    None,
                    ComponentType::Component,
                ),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };
        let schema_variant_id = *schema_variant.id();
        schema.set_ui_hidden(ctx, true).await?;

        schema_variant.set_link(ctx, Some("https://v1-22.docs.kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/".to_owned())).await?;

        let _metadata_prop = self
            .create_kubernetes_metadata_prop_for_namespace(
                ctx,
                root_prop.domain_prop_id,
                schema_variant_id,
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
            None,
            LeafKind::CodeGeneration,
            vec![LeafInput {
                location: LeafInputLocation::Domain,
                func_argument_id: *code_generation_func_argument.id(),
            }],
        )
        .await?;

        // Create sockets
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (external_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Kubernetes Namespace",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;

        schema_variant.finalize(ctx, None).await?;

        // Connect the "/root/si/name" field to the "/root/domain/metadata/name" field.
        let metadata_name_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "metadata", "name"])
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
        let si_name_prop = schema_variant
            .find_prop(ctx, &["root", "si", "name"])
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

    async fn create_kubernetes_metadata_prop_for_namespace(
        &self,
        ctx: &DalContext,
        parent_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<Prop> {
        let metadata_prop = self
            .create_prop(
                ctx,
                "metadata",
                PropKind::Object,
                None,
                Some(parent_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
                )),
                schema_variant_id,
            )
            .await?;

        {
            // TODO: add validation
            //validation: [
            //  {
            //    kind: ValidatorKind.Regex,
            //    regex: "^[A-Za-z0-9](?:[A-Za-z0-9-]{0,251}[A-Za-z0-9])?$",
            //    message: "Kubernetes names must be valid DNS subdomains",
            //    link:
            //      "https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#dns-subdomain-names",
            //  },
            //],

            let _name_prop = self
                .create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    None,
                    Some(*metadata_prop.id()),
                    Some(doc_url(
                        "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
                    )),
                    schema_variant_id,
                )
                .await?;
        }

        {
            let _generate_name_prop = self
                .create_prop(
                    ctx,
                    "generateName",
                    PropKind::String,
                    None,
                    Some(*metadata_prop.id()),
                    Some(doc_url(
                        "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
                    )),
                    schema_variant_id,
                )
                .await?;
        }

        {
            let _namespace_prop = self
                .create_prop(
                    ctx,
                    "namespace",
                    PropKind::String,
                    None,
                    Some(*metadata_prop.id()),
                    Some(doc_url(
                        "concepts/overview/working-with-objects/namespaces/",
                    )),
                    schema_variant_id,
                )
                .await?;
        }

        {
            let labels_prop = self
                .create_prop(
                    ctx,
                    "labels",
                    PropKind::Map,
                    None,
                    Some(*metadata_prop.id()),
                    Some(doc_url("concepts/overview/working-with-objects/labels/")),
                    schema_variant_id,
                )
                .await?;
            let _labels_value_prop = self
                .create_prop(
                    ctx,
                    "labelValue",
                    PropKind::String,
                    None,
                    Some(*labels_prop.id()),
                    Some(doc_url("concepts/overview/working-with-objects/labels/")),
                    schema_variant_id,
                )
                .await?;
        }

        {
            let annotations_prop = self
                .create_prop(
                    ctx,
                    "annotations",
                    PropKind::Map,
                    None, // How to specify it as a map of string values?
                    Some(*metadata_prop.id()),
                    Some(doc_url(
                        "concepts/overview/working-with-objects/annotations/",
                    )),
                    schema_variant_id,
                )
                .await?;
            let _annotations_value_prop = self
                .create_prop(
                    ctx,
                    "annotationValue",
                    PropKind::String,
                    None,
                    Some(*annotations_prop.id()),
                    Some(doc_url(
                        "concepts/overview/working-with-objects/annotations/",
                    )),
                    schema_variant_id,
                )
                .await?;
        }
        Ok(metadata_prop)
    }
}
