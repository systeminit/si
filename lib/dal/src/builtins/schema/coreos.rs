use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::func::argument::FuncArgument;
use crate::prototype_context::PrototypeContext;
use crate::schema::variant::definition::SchemaVariantDefinition;
use crate::socket::SocketArity;
use crate::{
    qualification_prototype::QualificationPrototypeContext, schema::SchemaUiMenu,
    AttributePrototype, AttributePrototypeArgument, AttributePrototypeError, AttributeReadContext,
    AttributeValue, BuiltinsError, BuiltinsResult, DalContext, DiagramKind, ExternalProvider, Func,
    FuncError, InternalProvider, QualificationPrototype, SchemaError, SchemaKind, SchemaVariant,
    StandardModel,
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

        let (schema, schema_variant, root_prop, maybe_prop_cache) = match self
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
        let code_generation_func_name = "si:generateButaneIgnition".to_owned();
        let code_generation_func =
            Func::find_by_attr(ctx, "name", &code_generation_func_name.clone())
                .await?
                .pop()
                .ok_or_else(|| SchemaError::FuncNotFound(code_generation_func_name.clone()))?;
        let code_generation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "domain", *code_generation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        code_generation_func_name.clone(),
                        "domain".to_string(),
                    )
                })?;
        let code_map_prop_id = SchemaVariant::add_code_generation(
            ctx,
            *code_generation_func.id(),
            *code_generation_func_argument.id(),
            *schema_variant.id(),
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

        // Qualification Prototype
        let qual_func_name = "si:qualificationButaneIsValidIgnition".to_string();
        let mut qual_funcs = Func::find_by_attr(ctx, "name", &qual_func_name).await?;
        let qual_func = qual_funcs
            .pop()
            .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
        let mut qual_prototype_context = QualificationPrototypeContext::new();
        qual_prototype_context.set_schema_variant_id(*schema_variant.id());
        let _ = QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

        // Wrap it up.
        self.finalize_schema_variant(ctx, &schema_variant, &root_prop)
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
        let transformation_func_name = "si:dockerImagesToButaneUnits".to_string();
        let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
            .await?
            .pop()
            .ok_or_else(|| FuncError::NotFoundByName(transformation_func_name.clone()))?;
        let images_arg =
            FuncArgument::find_by_name_for_func(ctx, "images", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        transformation_func_name.clone(),
                        "images".to_string(),
                    )
                })?;
        units_attribute_prototype
            .set_func_id(ctx, *transformation_func.id())
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *units_attribute_prototype.id(),
            *images_arg.id(),
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
        let transformation_func_name = "si:ignitionFromCodeMap".to_string();
        let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
            .await?
            .pop()
            .ok_or_else(|| FuncError::NotFoundByName(transformation_func_name.clone()))?;
        let transformation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "code", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        transformation_func_name.clone(),
                        "code".to_string(),
                    )
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
        user_data_external_provider_attribute_prototype
            .set_func_id(ctx, transformation_func.id())
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *user_data_external_provider_attribute_prototype_id,
            *transformation_func_argument.id(),
            *code_map_implicit_internal_provider.id(),
        )
        .await?;

        Ok(())
    }
}
