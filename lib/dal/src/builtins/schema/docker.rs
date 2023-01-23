use std::collections::HashMap;

use crate::schema::variant::leaves::LeafKind;
use crate::{action_prototype::ActionKind, schema::variant::leaves::LeafInputLocation};
use crate::{builtins::schema::MigrationDriver, schema::variant::leaves::LeafInput};
use crate::{
    component::ComponentKind, edit_field::widget::*, socket::SocketArity, ActionPrototype,
    ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, BuiltinsError, BuiltinsResult, DalContext, ExternalProvider, Func,
    InternalProvider, Prop, PropKind, SchemaError, SchemaVariant, StandardModel, WorkflowPrototype,
    WorkflowPrototypeContext,
};

// Reference: https://www.docker.com/company/newsroom/media-resources/
const DOCKER_NODE_COLOR: i64 = 0x4695E7;

impl MigrationDriver {
    pub async fn migrate_docker(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let ui_menu_category = "Docker";
        self.migrate_docker_hub_credential(ctx, ui_menu_category)
            .await?;
        self.migrate_docker_image(ctx, ui_menu_category).await?;
        Ok(())
    }

    async fn migrate_docker_hub_credential(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
    ) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                "Docker Hub Credential",
                Some(String::from("Credential")),
                ui_menu_category,
                ComponentKind::Credential,
                Some(DOCKER_NODE_COLOR),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        let mut secret_prop = Prop::new(ctx, "secret", PropKind::Integer, None).await?;
        secret_prop
            .set_parent_prop(ctx, root_prop.domain_prop_id)
            .await?;
        secret_prop
            .set_widget_kind(ctx, WidgetKind::SecretSelect)
            .await?;

        // Qualifications
        let (qualification_func_id, qualification_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:qualificationDockerHubLogin", "domain")
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

        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (_output_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Docker Hub Credential",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;

        schema_variant.finalize(ctx, None).await?;
        Ok(())
    }

    async fn migrate_docker_image(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
    ) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                "Docker Image",
                Some(String::from("Image")),
                ui_menu_category,
                ComponentKind::Standard,
                Some(DOCKER_NODE_COLOR),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        let image_prop = Prop::new(ctx, "image", PropKind::String, None).await?;
        image_prop
            .set_parent_prop(ctx, root_prop.domain_prop_id)
            .await?;

        // TODO: required, validate regex: "\\d+\\/(tcp|udp)", message: "invalid exposed port entry; must be [numeric]/(tcp|udp)",
        let exposed_ports_prop = Prop::new(ctx, "ExposedPorts", PropKind::Array, None).await?;
        exposed_ports_prop
            .set_parent_prop(ctx, root_prop.domain_prop_id)
            .await?;
        let exposed_port_prop = Prop::new(ctx, "ExposedPort", PropKind::String, None).await?;
        exposed_port_prop
            .set_parent_prop(ctx, *exposed_ports_prop.id())
            .await?;

        // TODO: we don't have a component to have their props, but we can manually rebuild the props from what we created in this schema variant
        // This means if someone updates this function the properties will be invalid
        let mut properties = HashMap::new();
        properties.insert("image".to_owned(), serde_json::json!(""));

        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (_docker_hub_credential_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
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
                "si:qualificationDockerImageNameInspect",
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
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *workflow_func.id(),
            serde_json::Value::Null,
            context,
            title,
        )
        .await?;

        let name = "refresh";
        let context = ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            name,
            ActionKind::Other,
            context,
        )
        .await?;

        Ok(())
    }
}
