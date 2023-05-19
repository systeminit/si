use crate::builtins::schema::{MigrationDriver, ValidationKind};
use crate::builtins::BuiltinsError;
use crate::component::ComponentKind;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafInput;
use crate::schema::variant::leaves::LeafKind;
use crate::socket::SocketArity;
use crate::validation::Validation;
use crate::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument,
    schema::variant::leaves::LeafInputLocation, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, AttributeValueError, BuiltinsResult, ComponentType, DalContext,
    ExternalProvider, Func, FuncBinding, FuncDescription, FuncDescriptionContents,
    InternalProvider, PropId, PropKind, SchemaError, SchemaVariant, SchemaVariantId, StandardModel,
};

// Documentation URL(s)
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";
const EC2_SECURITY_GROUP_PROPERTIES_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-security-group.html#aws-properties-ec2-security-group-properties";

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`AWS Security Group`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
    pub async fn migrate_aws_security_group(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let name = "Security Group";

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
                    ComponentType::Component,
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

        let description_prop = self
            .create_prop(
                ctx,
                "Description",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_PROPERTIES_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        self.create_validation(
            ctx,
            ValidationKind::Builtin(Validation::StringIsNotEmpty { value: None }),
            *description_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        let validate_fn_name = "si:awsSecurityGroupValidateDescription";
        let validate_fn = Func::find_by_attr(ctx, "name", &validate_fn_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(validate_fn_name.to_owned()))?;

        self.create_validation(
            ctx,
            ValidationKind::Custom(*validate_fn.id()),
            *description_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        let group_name_prop = self
            .create_prop(
                ctx,
                "GroupName",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_PROPERTIES_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        let vpc_id_prop = self
            .create_prop(
                ctx,
                "VpcId",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_SECURITY_GROUP_PROPERTIES_DOCS_URL.to_string()),
                schema_variant_id,
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
                schema_variant_id,
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
                schema_variant_id,
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
                schema_variant_id,
            )
            .await?;

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

        // Create Resource Prop Tree

        // Prop: /resource_value/GroupName
        let mut sg_group_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupName",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        sg_group_name_resource_prop
            .set_refers_to_prop_id(ctx, Some(*group_name_prop.id()))
            .await?;
        sg_group_name_resource_prop.set_default_diff(ctx).await?;

        // Prop: /resource_value/GroupId
        let _sg_group_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /resource_value/OwnerId
        let _sg_owner_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "OwnerId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /resource_value/VpcId
        let mut sg_vpc_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "VpcId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        sg_vpc_id_resource_prop
            .set_refers_to_prop_id(ctx, Some(*vpc_id_prop.id()))
            .await?;
        sg_vpc_id_resource_prop.set_default_diff(ctx).await?;

        // Prop: /resource_value/Description
        let mut sg_description_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Description",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        sg_description_resource_prop
            .set_refers_to_prop_id(ctx, Some(*description_prop.id()))
            .await?;
        sg_description_resource_prop.set_default_diff(ctx).await?;

        self.create_aws_tags_prop_tree(
            ctx,
            root_prop.resource_value_prop_id,
            schema_variant_id,
            None,
            None,
        )
        .await?;

        // Prop: /resource_value/IpPermissions
        let sg_ip_permissions_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpPermissions",
                PropKind::Array,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /resource_value/IpPermission
        let sg_ip_permission_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpPermission",
                PropKind::Object,
                Some(*sg_ip_permissions_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        self.create_aws_ip_permission_prop_tree(
            ctx,
            *sg_ip_permission_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        // Prop: /resource_value/IpPermissionsEgress
        let sg_ip_permissions_egress_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpPermissionsEgress",
                PropKind::Array,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        // Prop: /resource_value/IpPermissionEgress
        let sg_ip_permission_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpPermissionEgress",
                PropKind::Object,
                Some(*sg_ip_permissions_egress_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        self.create_aws_ip_permission_prop_tree(
            ctx,
            *sg_ip_permission_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        // Socket Creation
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

        let transformation_func_name = "si:awsSecurityGroupIdFromResource";
        let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(transformation_func_name.to_owned()))?;
        let transformation_func_id = *transformation_func.id();
        let (transformation_func_binding, transformation_func_binding_return_value) =
            FuncBinding::create_and_execute(ctx, serde_json::json!({}), transformation_func_id)
                .await?;

        let (security_group_id_external_provider, _output_socket) =
            ExternalProvider::new_with_socket(
                ctx,
                *schema.id(),
                *schema_variant.id(),
                "Security Group ID",
                None,
                transformation_func_id,
                *transformation_func_binding.id(),
                *transformation_func_binding_return_value.id(),
                SocketArity::Many,
                false,
            )
            .await?;

        // Code Generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(
                ctx,
                "si:generateAwsSecurityGroupJSON",
                "domain",
            )
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
        let qualification_func_name = "si:qualificationSecurityGroupCanCreate";
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

        // Wrap it up!
        schema_variant.finalize(ctx, None).await?;

        // Set Defaults
        self.set_default_value_for_prop(
            ctx,
            *aws_resource_type_prop.id(),
            serde_json::json!["security-group"],
        )
        .await?;

        // Get the SI Name Prop and internal provider
        let si_name_prop = schema_variant
            .find_prop(ctx, &["root", "si", "name"])
            .await?;
        let si_name_internal_provider = InternalProvider::find_for_prop(ctx, *si_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
            })?;

        // Create a custom function to set the default description for a SG
        let description_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext::default_with_prop(*description_prop.id()),
        )
        .await?
        .ok_or(AttributeValueError::Missing)?;

        let mut description_attribute_prototype = description_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;

        let default_description_func_name = "si:awsSecurityGroupDefaultDescription";
        let default_description_func =
            Func::find_by_attr(ctx, "name", &default_description_func_name)
                .await?
                .pop()
                .ok_or_else(|| {
                    SchemaError::FuncNotFound(default_description_func_name.to_owned())
                })?;

        let default_description_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "name", *default_description_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        default_description_func_name.to_owned(),
                        "name".to_string(),
                    )
                })?;

        description_attribute_prototype
            .set_func_id(ctx, default_description_func.id())
            .await?;

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *description_attribute_prototype.id(),
            *default_description_func_argument.id(),
            *si_name_internal_provider.id(),
        )
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

        // security_group_id to output socket
        let security_group_id_external_provider_attribute_prototype_id =
            security_group_id_external_provider
                .attribute_prototype_id()
                .ok_or_else(|| {
                    BuiltinsError::MissingAttributePrototypeForExternalProvider(
                        *security_group_id_external_provider.id(),
                    )
                })?;

        let security_group_id_internal_provider =
            InternalProvider::find_for_prop(ctx, root_prop.resource_prop_id)
                .await?
                .ok_or(BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                    root_prop.resource_prop_id,
                ))?;

        let func_argument =
            FuncArgument::find_by_name_for_func(ctx, "resource", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        "si:awsSecurityGroupIdFromResource".to_owned(),
                        "resource".to_owned(),
                    )
                })?;

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *security_group_id_external_provider_attribute_prototype_id,
            *func_argument.id(),
            *security_group_id_internal_provider.id(),
        )
        .await?;

        // region from input socket
        self.link_region_prop_to_explicit_internal_provider(
            ctx,
            region_prop.id(),
            identity_func_item.func_id,
            identity_func_item.func_argument_id,
            region_explicit_internal_provider.id(),
        )
        .await?;

        // Make GroupName take the value of /root/si/name
        let group_name_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext::default_with_prop(*group_name_prop.id()),
        )
        .await?
        .ok_or(AttributeValueError::Missing)?;
        let mut group_name_attribute_proto = group_name_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        group_name_attribute_proto
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *group_name_attribute_proto.id(),
            identity_func_item.func_argument_id,
            *si_name_internal_provider.id(),
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
                name: "Security Group Exists?".to_string(),
                success_description: Some("Security Group exists!".to_string()),
                failure_description: Some("This Security Group has not been created yet. Please run the fix above to create it!".to_string()),
                provider: Some("AWS".to_string()),
            },
        )
            .await?;

        self.add_deletion_confirmation(ctx, name, &schema_variant, Some("AWS"))
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn create_aws_security_group_rule_prop_tree(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
        group_id_prop_id: Option<PropId>,
        ip_protocol_prop_id: Option<PropId>,
        to_port_prop_id: Option<PropId>,
        from_port_prop_id: Option<PropId>,
        cidr_ip_prop_id: Option<PropId>,
        tags_array_prop_id: Option<PropId>,
        tag_item_prop_id: Option<PropId>,
    ) -> BuiltinsResult<()> {
        let _security_group_rule_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "SecurityGroupRuleId",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let mut security_group_rule_group_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupId",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;
        if group_id_prop_id.is_some() {
            security_group_rule_group_id_resource_prop
                .set_refers_to_prop_id(ctx, group_id_prop_id)
                .await?;
            security_group_rule_group_id_resource_prop
                .set_default_diff(ctx)
                .await?;
        }

        let _security_group_rule_group_owner_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupOwnerId",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let mut security_group_rule_ip_protocol_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpProtocol",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;
        if ip_protocol_prop_id.is_some() {
            security_group_rule_ip_protocol_resource_prop
                .set_refers_to_prop_id(ctx, ip_protocol_prop_id)
                .await?;
            security_group_rule_ip_protocol_resource_prop
                .set_default_diff(ctx)
                .await?;
        }

        let mut security_group_rule_from_port_resource_prop = self
            .create_hidden_prop(
                ctx,
                "FromPort",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;
        if from_port_prop_id.is_some() {
            security_group_rule_from_port_resource_prop
                .set_refers_to_prop_id(ctx, from_port_prop_id)
                .await?;
            security_group_rule_from_port_resource_prop
                .set_default_diff(ctx)
                .await?;
        }

        let mut security_group_rule_to_port_resource_prop = self
            .create_hidden_prop(
                ctx,
                "ToPort",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;
        if to_port_prop_id.is_some() {
            security_group_rule_to_port_resource_prop
                .set_refers_to_prop_id(ctx, to_port_prop_id)
                .await?;
            security_group_rule_to_port_resource_prop
                .set_default_diff(ctx)
                .await?;
        }

        let _security_group_rule_cidr_ipv6_resource_prop = self
            .create_hidden_prop(
                ctx,
                "CidrIpv6",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let mut security_group_rule_cidr_ipv4_resource_prop = self
            .create_hidden_prop(
                ctx,
                "CidrIpv4",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;
        if cidr_ip_prop_id.is_some() {
            security_group_rule_cidr_ipv4_resource_prop
                .set_refers_to_prop_id(ctx, cidr_ip_prop_id)
                .await?;
            security_group_rule_cidr_ipv4_resource_prop
                .set_default_diff(ctx)
                .await?;
        }

        let _security_group_rule_description_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Description",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        self.create_aws_tags_prop_tree(
            ctx,
            prop_id,
            schema_variant_id,
            tags_array_prop_id,
            tag_item_prop_id,
        )
        .await?;

        Ok(())
    }

    pub async fn create_aws_ip_permission_prop_tree(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        let _ip_perm_from_port_resource_prop = self
            .create_hidden_prop(
                ctx,
                "FromPort",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let _ip_perm_protocol_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpProtocol",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let _ip_perm_to_port_resource_prop = self
            .create_hidden_prop(
                ctx,
                "ToPort",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let ip_ranges_protocol_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpRanges",
                PropKind::Array,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let ip_range_protocol_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpRange",
                PropKind::Object,
                Some(*ip_ranges_protocol_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        self.create_ip_range_prop_tree(
            ctx,
            *ip_range_protocol_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        let ipv6_ranges_protocol_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Ipv6Ranges",
                PropKind::Array,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let ipv6_range_protocol_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Ipv6Range",
                PropKind::Object,
                Some(*ipv6_ranges_protocol_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        self.create_ip_range_prop_tree(
            ctx,
            *ipv6_range_protocol_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        let prefix_list_ids_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrefixListIds",
                PropKind::Array,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let prefix_list_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrefixListId",
                PropKind::Object,
                Some(*prefix_list_ids_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _prefix_list_id_description_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Description",
                PropKind::String,
                Some(*prefix_list_id_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _prefix_list_id_id_description_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrefixListId",
                PropKind::String,
                Some(*prefix_list_id_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        Ok(())
    }

    async fn create_ip_range_prop_tree(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        let _ip_range_description_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Description",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let _ip_range_cidr_ip_resource_prop = self
            .create_hidden_prop(
                ctx,
                "CidrIp",
                PropKind::String,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;
        Ok(())
    }
}
