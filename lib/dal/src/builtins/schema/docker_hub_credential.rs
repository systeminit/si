use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::schema::variant::leaves::LeafKind;
use crate::{builtins::schema::MigrationDriver, schema::variant::leaves::LeafInput, ComponentType};
use crate::{
    component::ComponentKind, property_editor::schema::WidgetKind, socket::SocketArity,
    BuiltinsError, BuiltinsResult, DalContext, ExternalProvider, Prop, PropKind, SchemaVariant,
    StandardModel,
};

impl MigrationDriver {
    pub async fn migrate_docker_hub_credential(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let (mut schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    "Docker Hub Credential",
                    Some("Credential"),
                    ui_menu_category,
                    node_color,
                    ComponentKind::Credential,
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
        schema.set_ui_hidden(ctx, true).await?;

        let mut secret_prop = Prop::new(
            ctx,
            "secret",
            PropKind::Integer,
            None,
            *schema_variant.id(),
            Some(root_prop.domain_prop_id),
        )
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
}
