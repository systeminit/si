use crate::builtins::schema::MigrationDriver;
use crate::builtins::BuiltinsError;
use crate::component::ComponentKind;
use crate::func::description::FuncDescription;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafKind;
use crate::socket::SocketArity;
use crate::SchemaVariant;
use crate::{
    action_prototype::ActionKind,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    FuncDescriptionContents,
};
use crate::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument, ActionPrototype,
    ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    BuiltinsResult, DalContext, ExternalProvider, Func, InternalProvider, PropKind, SchemaError,
    StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};

// Documentation URL(s)
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`AWS EIP`](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-eip.html).
    pub async fn migrate_aws_eip(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let name = "Elastic IP";
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    name,
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

        // Create Domain Prop Tree

        // Prop: /root/domain/tags
        let tags_map_prop = self
            .create_prop(
                ctx,
                "tags",
                PropKind::Map,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_TAG_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/domain/tags/tag
        let tags_map_item_prop = self
            .create_prop(
                ctx,
                "tag",
                PropKind::String,
                None,
                Some(*tags_map_prop.id()),
                Some(EC2_TAG_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/domain/awsResourceType
        let mut aws_resource_type_prop = self
            .create_prop(
                ctx,
                "awsResourceType",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                None,
                schema_variant_id,
            )
            .await?;
        aws_resource_type_prop.set_hidden(ctx, true).await?;

        // Prop: /root/domain/region
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

        // Create Resource Prop Tree

        // Prop: /root/resource_value/Domain
        let _eip_domain_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Domain",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/PublicIpv4Pool
        let _eip_public_ipv4_pool_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PublicIpv4Pool",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/InstanceId
        let _eip_instance_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "InstanceId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/NetworkInterfaceId
        let _eip_network_interface_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "NetworkInterfaceId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/AssociationId
        let _eip_association_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AssociationId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/NetworkInterfaceOwnerId
        let _eip_network_interface_owner_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "NetworkInterfaceOwnerId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/PublicIp
        let _eip_public_ip_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PublicIp",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/AllocationId
        let _eip_allocation_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AllocationId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/PrivateIpAddress
        let _eip_private_ip_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateIpAddress",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/resource_value/NetworkBorderGroup
        let mut eip_network_border_group_resource_prop = self
            .create_hidden_prop(
                ctx,
                "NetworkBorderGroup",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        eip_network_border_group_resource_prop
            .set_refers_to_prop_id(ctx, Some(*region_prop.id()))
            .await?;

        self.create_aws_tags_prop_tree(
            ctx,
            root_prop.resource_value_prop_id,
            schema_variant_id,
            Some(*tags_map_prop.id()),
            Some(*tags_map_item_prop.id()),
        )
        .await?;

        // Add code generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:generateAwsEipJSON", "domain")
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

        // Output Socket
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;
        let (_allocation_id_external_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Allocation ID",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;

        // Input Socket
        // PAUL: There are currently no options to create a different type of EIP
        // we may wat to allow passing in an address to specify coming from a pool
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
        let qualification_func_name = "si:qualificationEipCanCreate";
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

        // Set Defaults
        self.set_default_value_for_prop(
            ctx,
            *aws_resource_type_prop.id(),
            serde_json::json!["elastic-ip"],
        )
        .await?;

        let tags_map_attribute_read_context =
            AttributeReadContext::default_with_prop(*tags_map_prop.id());
        let tags_map_attribute_value =
            AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    tags_map_attribute_read_context,
                ))?;
        let tags_map_item_attribute_context = AttributeContextBuilder::new()
            .set_prop_id(*tags_map_item_prop.id())
            .to_context()?;
        let name_tags_item_attribute_value_id = AttributeValue::insert_for_context(
            ctx,
            tags_map_item_attribute_context,
            *tags_map_attribute_value.id(),
            None,
            Some("Name".to_string()),
        )
        .await?;

        // Connect props to providers.
        let si_name_prop = schema_variant
            .find_prop(ctx, &["root", "si", "name"])
            .await?;
        let si_name_internal_provider = InternalProvider::find_for_prop(ctx, *si_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
            })?;
        let name_tags_item_attribute_value =
            AttributeValue::get_by_id(ctx, &name_tags_item_attribute_value_id)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFound(
                    name_tags_item_attribute_value_id,
                ))?;
        let mut name_tags_item_attribute_prototype = name_tags_item_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        name_tags_item_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        let identity_arg =
            FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_item.func_id)
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        "identity".to_string(),
                        "identity".to_string(),
                    )
                })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *name_tags_item_attribute_prototype.id(),
            *identity_arg.id(),
            *si_name_internal_provider.id(),
        )
        .await?;

        // Connect the "region" prop to the "Region" explicit internal provider.
        self.link_region_prop_to_explicit_internal_provider(
            ctx,
            region_prop.id(),
            identity_func_item.func_id,
            identity_func_item.func_argument_id,
            region_explicit_internal_provider.id(),
        )
        .await?;

        let workflow_func_name = "si:awsEipCreateWorkflow";
        let workflow_func = Func::find_by_attr(ctx, "name", &workflow_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(workflow_func_name.to_owned()))?;
        let title = "Create Elastic IP";
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

        // Add confirmations.
        let confirmation_func_name = "si:confirmationResourceExists";
        let confirmation_func = Func::find_by_attr(ctx, "name", &confirmation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(confirmation_func_name.to_owned()))?;
        let confirmation_func_argument_name = "resource";
        let confirmation_func_argument = FuncArgument::find_by_name_for_func(
            ctx,
            confirmation_func_argument_name,
            *confirmation_func.id(),
        )
        .await?
        .ok_or_else(|| {
            BuiltinsError::BuiltinMissingFuncArgument(
                confirmation_func_name.to_string(),
                confirmation_func_argument_name.to_string(),
            )
        })?;
        SchemaVariant::add_leaf(
            ctx,
            *confirmation_func.id(),
            *schema_variant.id(),
            None,
            LeafKind::Confirmation,
            vec![LeafInput {
                location: LeafInputLocation::Resource,
                func_argument_id: *confirmation_func_argument.id(),
            }],
        )
        .await
        .expect("could not add leaf");
        FuncDescription::new(
            ctx,
            *confirmation_func.id(),
            *schema_variant.id(),
            FuncDescriptionContents::Confirmation {
                name: "Elastic IP Exists?".to_string(),
                success_description: Some("Elastic IP exists!".to_string()),
                failure_description: Some("This Elastic IP has not been created yet. Please run the fix above to create it!".to_string()),
                provider: Some("AWS".to_string()),
            },
        )
            .await?;

        // Add delete confirmations.
        self.add_deletion_confirmation_and_workflow(
            ctx,
            name,
            &schema_variant,
            Some("AWS"),
            "si:awsEipDeleteWorkflow",
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
            "create",
            ActionKind::Create,
            context,
        )
        .await?;

        let eip_refresh_workflow_func_name = "si:awsEipRefreshWorkflow";
        let eip_refresh_workflow_func =
            Func::find_by_attr(ctx, "name", &eip_refresh_workflow_func_name)
                .await?
                .pop()
                .ok_or_else(|| {
                    SchemaError::FuncNotFound(eip_refresh_workflow_func_name.to_owned())
                })?;
        let title = "Refresh Elastic IP Resource";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *eip_refresh_workflow_func.id(),
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

        Ok(())
    }
}
