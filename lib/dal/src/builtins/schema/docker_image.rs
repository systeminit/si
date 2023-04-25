use std::collections::HashMap;

use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::schema::variant::leaves::LeafKind;
use crate::{builtins::schema::MigrationDriver, schema::variant::leaves::LeafInput};
use crate::{
    component::ComponentKind, socket::SocketArity, AttributePrototypeArgument,
    AttributeReadContext, AttributeValue, AttributeValueError, BuiltinsError, BuiltinsResult,
    DalContext, ExternalProvider, Func, InternalProvider, Prop, PropKind, SchemaError,
    SchemaVariant, StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};

impl MigrationDriver {
    pub async fn migrate_docker_image(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    "Docker Image",
                    Some("Image"),
                    ui_menu_category,
                    node_color,
                    ComponentKind::Standard,
                    None,
                    None,
                ),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        let image_prop = Prop::new(
            ctx,
            "image",
            PropKind::String,
            None,
            *schema_variant.id(),
            Some(root_prop.domain_prop_id),
        )
        .await?;

        // TODO: required, validate regex: "\\d+\\/(tcp|udp)", message: "invalid exposed port entry; must be [numeric]/(tcp|udp)",
        let exposed_ports_prop = Prop::new(
            ctx,
            "ExposedPorts",
            PropKind::Array,
            None,
            *schema_variant.id(),
            Some(root_prop.domain_prop_id),
        )
        .await?;
        let _exposed_port_prop = Prop::new(
            ctx,
            "ExposedPort",
            PropKind::String,
            None,
            *schema_variant.id(),
            Some(*exposed_ports_prop.id()),
        )
        .await?;

        // TODO: we don't have a component to have their props, but we can manually rebuild the props from what we created in this schema variant
        // This means if someone updates this function the properties will be invalid
        let mut properties = HashMap::new();
        properties.insert("image".to_owned(), serde_json::json!(""));

        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (
            _docker_hub_credential_explicit_internal_provider,
            mut docker_hub_credential_input_socket,
        ) = InternalProvider::new_explicit_with_socket(
            ctx,
            *schema_variant.id(),
            "Docker Hub Credential",
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;
        docker_hub_credential_input_socket
            .set_ui_hidden(ctx, true)
            .await?;

        let (docker_image_external_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Container Image",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;

        let (exposed_ports_external_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Exposed Ports",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;

        // Qualifications
        let (qualification_func_id, qualification_func_argument_id) = self
            .find_func_and_single_argument_by_names(
                ctx,
                "si:qualificationDockerImageExists",
                "domain",
            )
            .await?;
        SchemaVariant::add_leaf(
            ctx,
            qualification_func_id,
            *schema_variant.id(),
            None,
            LeafKind::Qualification,
            vec![LeafInput {
                location: LeafInputLocation::Domain,
                func_argument_id: qualification_func_argument_id,
            }],
        )
        .await?;

        schema_variant.finalize(ctx, None).await?;

        // Connect the "/root/si/name" field to the "/root/domain/image" field.
        let image_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext::default_with_prop(*image_prop.id()),
        )
        .await?
        .ok_or(AttributeValueError::Missing)?;
        let mut image_attribute_prototype =
            image_attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(AttributeValueError::MissingAttributePrototype)?;
        image_attribute_prototype
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
            *image_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *si_name_internal_provider.id(),
        )
        .await?;

        // Connect "/root" to the external provider.
        let root_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, root_prop.prop_id)
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
            identity_func_item.func_argument_id,
            *root_implicit_internal_provider.id(),
        )
        .await?;

        // Connect "/root/domain/ExposedPorts" to the external provider.
        let exposed_props_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, *exposed_ports_prop.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::ImplicitInternalProviderNotFoundForProp(*exposed_ports_prop.id())
                })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *exposed_ports_external_provider
                .attribute_prototype_id()
                .ok_or_else(|| {
                    BuiltinsError::MissingAttributePrototypeForExternalProvider(
                        *exposed_ports_external_provider.id(),
                    )
                })?,
            identity_func_item.func_argument_id,
            *exposed_props_implicit_internal_provider.id(),
        )
        .await?;

        let workflow_func_name = "si:dockerImageRefreshWorkflow";
        let workflow_func = Func::find_by_attr(ctx, "name", &workflow_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(workflow_func_name.to_owned()))?;
        let title = "Refresh Docker Image";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let _workflow_prototype = WorkflowPrototype::new(
            ctx,
            *workflow_func.id(),
            serde_json::Value::Null,
            context,
            title,
        )
        .await?;

        // TODO(paulo): restore this when the following PR is merged: https://github.com/systeminit/si/pull/1876
        // It gives us the ability to check if the fix flow has been run
        // Which allows us to identify if a resource has actually been created in real-life, or if
        // we are just passively monitoring it, like with AMI, Docker Image and Region
        // By doing that we can avoid setting needs_destroy for passive components
        /*
        let context = ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            "refresh",
            ActionKind::Refresh,
            context,
        )
        .await?;
        */

        Ok(())
    }
}
