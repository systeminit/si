use crate::builtins::schema::MigrationDriver;
use crate::builtins::BuiltinsError;
use crate::component::ComponentKind;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafInput;
use crate::schema::variant::leaves::LeafKind;
use crate::socket::SocketArity;
use crate::validation::Validation;
use crate::{
    action_prototype::ActionKind, schema::variant::leaves::LeafInputLocation, FuncDescription,
    FuncDescriptionContents,
};
use crate::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument, ActionPrototype,
    ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    BuiltinsResult, DalContext, Func, InternalProvider, PropKind, SchemaError, SchemaVariant,
    StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};

// Documentation URL(s)
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";
const EC2_SECURITY_GROUP_EGRESS_PROPERTIES_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-security-group-egress.html#aws-properties-ec2-security-group-egress-properties";
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";

// Dataset(s)
const INGRESS_EGRESS_PROTOCOLS: &[&str; 3] = &["tcp", "udp", "icmp"];

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`AWS Egress`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
    pub async fn migrate_aws_egress(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let name = "Egress";
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

        // Prop Creation
        let group_id_prop = self
            .create_prop(
                ctx,
                "GroupId",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_EGRESS_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        let protocol_prop = self
            .create_prop(
                ctx,
                "IpProtocol",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_EGRESS_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        let expected = INGRESS_EGRESS_PROTOCOLS
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>();
        self.create_validation(
            ctx,
            Validation::StringInStringArray {
                value: None,
                expected,
                display_expected: true,
            },
            *protocol_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        let from_port_prop = self
            .create_prop(
                ctx,
                "FromPort",
                PropKind::Integer,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_EGRESS_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        self.create_validation(
            ctx,
            Validation::IntegerIsBetweenTwoIntegers {
                value: None,
                lower_bound: -1,
                upper_bound: 65537,
            },
            *from_port_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        let to_port_prop = self
            .create_prop(
                ctx,
                "ToPort",
                PropKind::Integer,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_EGRESS_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        self.create_validation(
            ctx,
            Validation::IntegerIsBetweenTwoIntegers {
                value: None,
                lower_bound: -1,
                upper_bound: 65537,
            },
            *to_port_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        let cidr_prop = self
            .create_prop(
                ctx,
                "CidrIp",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_EGRESS_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        self.create_validation(
            ctx,
            Validation::StringIsValidIpAddr { value: None },
            *cidr_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        let region_prop = self
            .create_prop(
                ctx,
                "region",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(AWS_REGIONS_DOCS_URL.to_string()),
            )
            .await?;

        let aws_resource_type_prop = self
            .create_prop(
                ctx,
                "awsResourceType",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                None,
            )
            .await?;

        let tags_map_prop = self
            .create_prop(
                ctx,
                "tags",
                PropKind::Map,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_TAG_DOCS_URL.to_string()),
            )
            .await?;

        let tags_map_item_prop = self
            .create_prop(
                ctx,
                "tag",
                PropKind::String,
                None,
                Some(*tags_map_prop.id()),
                Some(EC2_TAG_DOCS_URL.to_string()),
            )
            .await?;

        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        // Input Socket
        let (group_id_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Security Group ID",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await?;

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

        // Code Generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:generateAwsEgressJSON", "domain")
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

        // Qualifications
        let qualification_func_name = "si:qualificationEgressCanCreate";
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
            serde_json::json!["security-group-rule"],
        )
        .await?;
        self.set_default_value_for_prop(ctx, *protocol_prop.id(), serde_json::json!["tcp"])
            .await?;

        // Bind sockets to providers
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

        let si_name_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "name")
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

        // region from input socket
        let region_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*region_prop.id());
        let region_attribute_value =
            AttributeValue::find_for_context(ctx, region_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    region_attribute_value_read_context,
                ))?;
        let mut region_attribute_prototype = region_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        region_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *region_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *region_explicit_internal_provider.id(),
        )
        .await?;

        // security group id from input socket
        let group_id_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*group_id_prop.id());
        let group_id_attribute_value =
            AttributeValue::find_for_context(ctx, group_id_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    group_id_attribute_value_read_context,
                ))?;
        let mut group_id_attribute_prototype = group_id_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        group_id_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *group_id_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *group_id_internal_provider.id(),
        )
        .await?;

        let workflow_func_name = "si:awsEgressCreateWorkflow";
        let workflow_func = Func::find_by_attr(ctx, "name", &workflow_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(workflow_func_name.to_owned()))?;
        let title = "Create Egress";
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
                name: "Egress Exists?".to_string(),
                success_description: Some("Egress exists!".to_string()),
                failure_description: Some("This Egress rule has not been created yet. Please run the fix above to create it!".to_string()),
                provider: Some("AWS".to_string()),
            },
        )
            .await?;

        let name = "create";
        let context = ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            name,
            ActionKind::Create,
            context,
        )
        .await?;

        let egress_refresh_workflow_func_name = "si:awsEgressRefreshWorkflow";
        let egress_refresh_workflow_func =
            Func::find_by_attr(ctx, "name", &egress_refresh_workflow_func_name)
                .await?
                .pop()
                .ok_or_else(|| {
                    SchemaError::FuncNotFound(egress_refresh_workflow_func_name.to_owned())
                })?;
        let title = "Refresh Egress's Resource";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *egress_refresh_workflow_func.id(),
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

        self.add_deletion_confirmation_and_workflow(
            ctx,
            name,
            &schema_variant,
            Some("AWS"),
            "si:awsEgressDeleteWorkflow",
        )
        .await?;

        Ok(())
    }
}
