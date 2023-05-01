use crate::schema::variant::definition::{
    SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
};
use crate::schema::variant::leaves::LeafKind;
use crate::{
    builtins::schema::MigrationDriver,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    ExternalProvider,
};
use crate::{
    AttributePrototype, AttributePrototypeArgument, AttributePrototypeError, AttributeReadContext,
    AttributeValue, BuiltinsError, BuiltinsResult, DalContext, InternalProvider, SchemaVariant,
    StandardModel,
};

// Definitions
const BUTANE_DEFINITION: &str = include_str!("definitions/core_os_butane.json");
const BUTANE_DEFINITION_METADATA: &str = include_str!("definitions/core_os_butane.metadata.json");

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`Butane`](https://coreos.github.io/butane/).
    pub async fn migrate_coreos_butane(
        &self,
        ctx: &DalContext,
        _ui_menu_category: &str,
        _node_color: &str,
    ) -> BuiltinsResult<()> {
        let definition: SchemaVariantDefinitionJson = serde_json::from_str(BUTANE_DEFINITION)?;
        let metadata: SchemaVariantDefinitionMetadataJson =
            serde_json::from_str(BUTANE_DEFINITION_METADATA)?;

        SchemaVariantDefinition::new_from_structs(ctx, metadata.clone(), definition.clone())
            .await?;

        let (
            _schema,
            mut schema_variant,
            root_prop,
            maybe_prop_cache,
            _explicit_internal_providers,
            _external_providers,
        ) = match self
            .create_schema_and_variant(ctx, metadata, Some(definition))
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        // Code generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:generateButaneIgnition", "domain")
            .await?;
        let (code_map_prop_id, _) = SchemaVariant::add_leaf(
            ctx,
            code_generation_func_id,
            *schema_variant.id(),
            None,
            LeafKind::CodeGeneration,
            vec![LeafInput {
                location: LeafInputLocation::Domain,
                func_argument_id: code_generation_func_argument_id,
            }],
        )
        .await?;

        // Qualifications
        let (qualification_func_id, qualification_func_argument_id) = self
            .find_func_and_single_argument_by_names(
                ctx,
                "si:qualificationButaneIsValidIgnition",
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

        // Wrap it up.
        schema_variant.finalize(ctx, None).await?;

        // Collect the props we need.
        let prop_cache = maybe_prop_cache
            .ok_or_else(|| BuiltinsError::PropCacheNotFound(*schema_variant.id()))?;
        let variant_prop_id = prop_cache.get("variant", root_prop.domain_prop_id)?;
        let version_prop_id = prop_cache.get("version", root_prop.domain_prop_id)?;
        let systemd_prop_id = prop_cache.get("systemd", root_prop.domain_prop_id)?;
        let units_prop_id = prop_cache.get("units", systemd_prop_id)?;

        let docker_image_explicit_internal_provider_name = "Container Image".to_string();
        let docker_image_explicit_internal_provider =
            InternalProvider::find_explicit_for_schema_variant_and_name(
                ctx,
                *schema_variant.id(),
                &docker_image_explicit_internal_provider_name,
            )
            .await?
            .ok_or_else(|| {
                BuiltinsError::ExplicitInternalProviderNotFound(
                    docker_image_explicit_internal_provider_name,
                )
            })?;

        let user_data_external_provider_name = "User Data".to_string();
        let user_data_external_provider = ExternalProvider::find_for_schema_variant_and_name(
            ctx,
            *schema_variant.id(),
            &user_data_external_provider_name,
        )
        .await?
        .ok_or_else(|| {
            BuiltinsError::ExternalProviderNotFound(user_data_external_provider_name.to_string())
        })?;

        // Set default values after finalization.
        self.set_default_value_for_prop(ctx, variant_prop_id, serde_json::json!["fcos"])
            .await?;
        self.set_default_value_for_prop(ctx, version_prop_id, serde_json::json!["1.4.0"])
            .await?;

        // Enable connections from the "Container Image" explicit internal provider to the
        // "/root/domain/systemd/units/" field. We need to use the appropriate function with and name
        // the argument "images".
        let units_attribute_value_read_context =
            AttributeReadContext::default_with_prop(units_prop_id);
        let units_attribute_value =
            AttributeValue::find_for_context(ctx, units_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    units_attribute_value_read_context,
                ))?;
        let mut units_attribute_prototype =
            units_attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        let (transformation_func_id, transformation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:dockerImagesToButaneUnits", "images")
            .await?;
        units_attribute_prototype
            .set_func_id(ctx, transformation_func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *units_attribute_prototype.id(),
            transformation_func_argument_id,
            *docker_image_explicit_internal_provider.id(),
        )
        .await?;

        // Connect the "/root/code" map to the external provider and use a transformation function
        // to grab the ignition data (aws ec2 user data) out of the map.
        let code_map_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, code_map_prop_id)
                .await?
                .ok_or({
                    BuiltinsError::ImplicitInternalProviderNotFoundForProp(code_map_prop_id)
                })?;
        let user_data_external_provider_attribute_prototype_id = user_data_external_provider
            .attribute_prototype_id()
            .ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(
                    *user_data_external_provider.id(),
                )
            })?;
        let mut user_data_external_provider_attribute_prototype =
            AttributePrototype::get_by_id(ctx, user_data_external_provider_attribute_prototype_id)
                .await?
                .ok_or_else(|| {
                    AttributePrototypeError::NotFound(
                        *user_data_external_provider_attribute_prototype_id,
                        *ctx.visibility(),
                    )
                })?;
        let (transformation_func_id, transformation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:ignitionFromCodeMap", "code")
            .await?;
        user_data_external_provider_attribute_prototype
            .set_func_id(ctx, &transformation_func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *user_data_external_provider_attribute_prototype_id,
            transformation_func_argument_id,
            *code_map_implicit_internal_provider.id(),
        )
        .await?;

        Ok(())
    }
}
