use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::schema::variant::definition::SchemaVariantDefinition;
use crate::schema::variant::leaves::LeafKind;
use crate::socket::SocketArity;
use crate::{
    schema::SchemaUiMenu, AttributePrototype, AttributePrototypeArgument, AttributePrototypeError,
    AttributeReadContext, AttributeValue, BuiltinsError, BuiltinsResult, DalContext, DiagramKind,
    ExternalProvider, InternalProvider, SchemaError, SchemaKind, SchemaVariant, StandardModel,
};

// Definitions
const BUTANE_DEFINITION: &str = include_str!("definitions/coreos/butane.json");

// Reference: https://getfedora.org/
const COREOS_NODE_COLOR: i64 = 0xE26B70;

impl MigrationDriver {
    pub async fn migrate_coreos(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_butane(ctx).await?;
        Ok(())
    }

    /// A [`Schema`](crate::Schema) migration for [`Butane`](https://coreos.github.io/butane/).
    async fn migrate_butane(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let definition: SchemaVariantDefinition = serde_json::from_str(BUTANE_DEFINITION)?;

        let (schema, mut schema_variant, root_prop, maybe_prop_cache) = match self
            .create_schema_and_variant(
                ctx,
                "Butane",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(COREOS_NODE_COLOR),
                Some(definition),
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        // Diagram and UI Menu
        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
        let ui_menu = SchemaUiMenu::new(ctx, "Butane", "CoreOS", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        // Code generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:generateButaneIgnition", "domain")
            .await?;
        let code_map_prop_id = SchemaVariant::add_leaf(
            ctx,
            code_generation_func_id,
            code_generation_func_argument_id,
            *schema_variant.id(),
            LeafKind::CodeGeneration,
        )
        .await?;

        // Sockets
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (user_data_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "User Data",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
        output_socket.set_color(ctx, Some(0xd61e8c)).await?;

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
            qualification_func_argument_id,
            *schema_variant.id(),
            LeafKind::Qualification,
        )
        .await?;

        // Wrap it up.
        self.finalize_schema_variant(ctx, &mut schema_variant, &root_prop)
            .await?;

        // Collect the props we need.
        let prop_cache = maybe_prop_cache
            .ok_or_else(|| BuiltinsError::PropCacheNotFound(*schema_variant.id()))?;
        let variant_prop_id = prop_cache.get("variant", root_prop.domain_prop_id)?;
        let version_prop_id = prop_cache.get("version", root_prop.domain_prop_id)?;
        let systemd_prop_id = prop_cache.get("systemd", root_prop.domain_prop_id)?;
        let units_prop_id = prop_cache.get("units", systemd_prop_id)?;

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
