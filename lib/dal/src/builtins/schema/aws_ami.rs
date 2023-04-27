use crate::{
    builtins::schema::MigrationDriver,
    builtins::BuiltinsError,
    component::ComponentKind,
    schema::variant::definition::SchemaVariantDefinitionMetadataJson,
    schema::variant::leaves::{LeafInput, LeafInputLocation, LeafKind},
    socket::SocketArity,
    AttributePrototypeArgument, BuiltinsResult, DalContext, ExternalProvider, Func, FuncArgument,
    FuncBinding, InternalProvider, PropKind, SchemaError, SchemaVariant, StandardModel,
};

// Documentation URL(s)
const AMI_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/finding-an-ami.html";
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`AWS AMI`](https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html).
    pub async fn migrate_aws_ami(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    "AMI",
                    None::<&str>,
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
        let schema_variant_id = *schema_variant.id();

        // Prop and validation creation
        let _image_id_prop = self
            .create_prop(
                ctx,
                "ImageId",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(AMI_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        // self.create_validation(
        //     ctx,
        //     Validation::StringHasPrefix {
        //         value: None,
        //         expected: "ami-".to_string(),
        //     },
        //     *image_id_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        // )
        // .await?;

        let region_prop = self
            .create_prop(
                ctx,
                "region",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(AWS_REGIONS_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        let _executable_users_prop = self
            .create_prop(
                ctx,
                "ExecutableUsers",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                None,
                schema_variant_id,
            )
            .await?;

        let _owners_prop = self
            .create_prop(
                ctx,
                "Owners",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                None,
                schema_variant_id,
            )
            .await?;

        let filters_prop = self
            .create_prop(
                ctx,
                "Filters",
                PropKind::Array,
                None,
                Some(root_prop.domain_prop_id),
                None,
                schema_variant_id,
            )
            .await?;

        let filter_prop = self
            .create_prop(
                ctx,
                "Filter",
                PropKind::Object,
                None,
                Some(*filters_prop.id()),
                None,
                schema_variant_id,
            )
            .await?;

        let _name_prop = self
            .create_prop(
                ctx,
                "Name",
                PropKind::String,
                None,
                Some(*filter_prop.id()),
                None,
                schema_variant_id,
            )
            .await?;

        let _value_prop = self
            .create_prop(
                ctx,
                "Value",
                PropKind::String,
                None,
                Some(*filter_prop.id()),
                None,
                schema_variant_id,
            )
            .await?;

        let _use_most_recent_prop = self
            .create_prop(
                ctx,
                "UseMostRecent",
                PropKind::Boolean,
                None,
                Some(root_prop.domain_prop_id),
                None,
                schema_variant_id,
            )
            .await?;

        // Code Generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:generateAwsAmiJSON", "domain")
            .await?;
        SchemaVariant::add_leaf(
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

        // Sockets
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (region_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Region",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await?;

        // Qualifications
        let qualification_func_name = "si:qualificationAmiExists";
        let (qualification_func_id, qualification_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, qualification_func_name, "domain")
            .await?;
        let code_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "code", qualification_func_id)
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        qualification_func_name.to_string(),
                        "code".to_string(),
                    )
                })?;
        SchemaVariant::add_leaf(
            ctx,
            qualification_func_id,
            *schema_variant.id(),
            None,
            LeafKind::Qualification,
            vec![
                LeafInput {
                    location: LeafInputLocation::Domain,
                    func_argument_id: qualification_func_argument_id,
                },
                LeafInput {
                    location: LeafInputLocation::Code,
                    func_argument_id: *code_func_argument.id(),
                },
            ],
        )
        .await?;

        // Wrap it up.
        schema_variant.finalize(ctx, None).await?;

        let transformation_func_name = "si:awsAmiImageIdFromApi";
        let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(transformation_func_name.to_owned()))?;
        let transformation_func_id = *transformation_func.id();
        let (transformation_func_binding, transformation_func_binding_return_value) =
            FuncBinding::create_and_execute(ctx, serde_json::json!({}), transformation_func_id)
                .await?;

        let (image_id_external_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Image ID",
            None,
            transformation_func_id,
            *transformation_func_binding.id(),
            *transformation_func_binding_return_value.id(),
            SocketArity::Many,
            false,
        )
        .await?;

        let image_id_external_provider_attribute_prototype_id = image_id_external_provider
            .attribute_prototype_id()
            .ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(
                    *image_id_external_provider.id(),
                )
            })?;

        let domain_internal_provider =
            InternalProvider::find_for_prop(ctx, root_prop.domain_prop_id)
                .await?
                .ok_or(BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                    root_prop.domain_prop_id,
                ))?;

        let code_internal_provider = InternalProvider::find_for_prop(ctx, root_prop.code_prop_id)
            .await?
            .ok_or(BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                root_prop.code_prop_id,
            ))?;

        let domain_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "domain", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        "si:awsAmiImageIdFromApi".to_owned(),
                        "domain".to_owned(),
                    )
                })?;

        let code_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "code", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        "si:awsAmiImageIdFromApi".to_owned(),
                        "code".to_owned(),
                    )
                })?;

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *image_id_external_provider_attribute_prototype_id,
            *domain_func_argument.id(),
            *domain_internal_provider.id(),
        )
        .await?;

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *image_id_external_provider_attribute_prototype_id,
            *code_func_argument.id(),
            *code_internal_provider.id(),
        )
        .await?;

        // Connect the props to providers.
        /* With the transformation func above, we don't *also* want this link!
        let external_provider_attribute_prototype_id = image_id_external_provider
            .attribute_prototype_id()
            .ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(
                    *image_id_external_provider.id(),
                )
            })?;
        let image_id_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, *image_id_prop.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::ImplicitInternalProviderNotFoundForProp(*image_id_prop.id())
                })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *external_provider_attribute_prototype_id,
            identity_func_item.func_argument_id,
            *image_id_implicit_internal_provider.id(),
        )
        .await?;
        */

        self.link_region_prop_to_explicit_internal_provider(
            ctx,
            region_prop.id(),
            identity_func_item.func_id,
            identity_func_item.func_argument_id,
            region_explicit_internal_provider.id(),
        )
        .await?;

        // TODO(paulo): restore this when the following PR is merged: https://github.com/systeminit/si/pull/1876
        // It gives us the ability to check if the fix flow has been run
        // Which allows us to identify if a resource has actually been created in real-life, or if
        // we are just passively monitoring it, like with AMI, Docker Image and Region
        // By doing that we can avoid setting needs_destroy for passive components
        /*
        let workflow_func_name = "si:awsAmiRefreshWorkflow";
        let workflow_func = Func::find_by_attr(ctx, "name", &workflow_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(workflow_func_name.to_owned()))?;
        let title = "Refresh AMI";
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
