use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::func::argument::FuncArgument;
use crate::prototype_context::PrototypeContext;
use crate::socket::SocketArity;
use crate::{
    qualification_prototype::QualificationPrototypeContext, schema::SchemaUiMenu, AttributeContext,
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, BuiltinsError,
    BuiltinsResult, CodeGenerationPrototype, CodeLanguage, DalContext, DiagramKind,
    ExternalProvider, Func, FuncError, InternalProvider, PropKind, QualificationPrototype,
    SchemaError, SchemaKind, StandardModel,
};

// Reference: https://getfedora.org/
const COREOS_NODE_COLOR: i64 = 0xE26B70;
const BUTANE_DOCS_FCOS_1_4_URL: &str = "https://coreos.github.io/butane/config-fcos-v1_4/";

impl MigrationDriver {
    pub async fn migrate_coreos(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_butane(ctx).await?;
        Ok(())
    }

    /// A [`Schema`](crate::Schema) migration for [`Butane`](https://coreos.github.io/butane/).
    async fn migrate_butane(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, schema_variant, root_prop) = match self
            .create_schema_and_variant(
                ctx,
                "Butane",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(COREOS_NODE_COLOR),
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        let mut attribute_context_builder = AttributeContext::builder();
        attribute_context_builder
            .set_schema_id(*schema.id())
            .set_schema_variant_id(*schema_variant.id());

        // Diagram and UI Menu
        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
        let ui_menu = SchemaUiMenu::new(ctx, "Butane", "CoreOS", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        // Prop creation
        let variant_prop = self
            .create_prop(
                ctx,
                "variant",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
            )
            .await?;
        let version_prop = self
            .create_prop(
                ctx,
                "version",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
            )
            .await?;
        let systemd_prop = self
            .create_prop(
                ctx,
                "systemd",
                PropKind::Object,
                None,
                Some(root_prop.domain_prop_id),
                Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
            )
            .await?;
        let units_prop = self
            .create_prop(
                ctx,
                "units",
                PropKind::Array,
                None,
                Some(*systemd_prop.id()),
                Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
            )
            .await?;
        let unit_prop = self
            .create_prop(
                ctx,
                "unit",
                PropKind::Object,
                None,
                Some(*units_prop.id()),
                Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
            )
            .await?;
        {
            let _units_name_prop = self
                .create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    None,
                    Some(*unit_prop.id()),
                    Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
                )
                .await?;
            let _units_enabled_prop = self
                .create_prop(
                    ctx,
                    "enabled",
                    PropKind::Boolean,
                    None,
                    Some(*unit_prop.id()),
                    Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
                )
                .await?;
            let _units_contents_prop = self
                .create_prop(
                    ctx,
                    "contents",
                    PropKind::String,
                    None,
                    Some(*unit_prop.id()),
                    Some(BUTANE_DOCS_FCOS_1_4_URL.to_string()),
                )
                .await?;
        }

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
        let ignition_code_generation_prototype = CodeGenerationPrototype::new(
            ctx,
            *code_generation_func.id(),
            *code_generation_func_argument.id(),
            *schema_variant.id(),
            CodeLanguage::Json,
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
        schema_variant.finalize(ctx).await?;

        // Set default values after finalization.
        self.set_default_value_for_prop(
            ctx,
            *variant_prop.id(),
            *schema.id(),
            *schema_variant.id(),
            serde_json::json!["fcos"],
        )
        .await?;
        self.set_default_value_for_prop(
            ctx,
            *version_prop.id(),
            *schema.id(),
            *schema_variant.id(),
            serde_json::json!["1.4.0"],
        )
        .await?;

        // Add the ability to use docker image as an input.
        let base_attribute_read_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            ..AttributeReadContext::default()
        };

        // Enable connections from the "Container Image" explicit internal provider to the
        // "/root/domain/systemd/units/" field. We need to use the appropriate function with and name
        // the argument "images".
        let units_attribute_value_read_context = AttributeReadContext {
            prop_id: Some(*units_prop.id()),
            ..base_attribute_read_context
        };
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

        // FIXME(nick,jacob): when setting a complex object, implicit internal providers of child props
        // must be updated. Currently, that does not happen. Thus, code generation functions return
        // strings for now (just the code) instead of the code _and_ the format. Moreover, we collect
        // the value for the implicit internal provider for the code string prop instead of the object
        // prop as a result.
        let code_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, ignition_code_generation_prototype.code_prop_id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                        ignition_code_generation_prototype.code_prop_id(),
                    )
                })?;
        let user_data_external_provider_attribute_prototype_id = *user_data_external_provider
            .attribute_prototype_id()
            .ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(
                    *user_data_external_provider.id(),
                )
            })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            user_data_external_provider_attribute_prototype_id,
            identity_func_item.func_argument_id,
            *code_implicit_internal_provider.id(),
        )
        .await?;

        Ok(())
    }
}
