use crate::builtins::schema::{MigrationDriver, ValidationKind};
use crate::builtins::BuiltinsError;
use crate::component::ComponentKind;
use crate::func::description::FuncDescription;
use crate::property_editor::schema::WidgetKind;
use crate::property_editor::SelectWidgetOption;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafKind;
use crate::socket::SocketArity;
use crate::validation::Validation;
use crate::{
    action_prototype::ActionKind,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    FuncDescriptionContents,
};
use crate::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument, ActionPrototype,
    ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    BuiltinsResult, DalContext, Func, InternalProvider, PropKind, SchemaError, StandardModel,
    WorkflowPrototype, WorkflowPrototypeContext,
};
use crate::{PropId, SchemaVariant, SchemaVariantId};

// Documentation URL(s)
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";
const EC2_INSTANCE_TYPES_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instance-types.html";
const AMI_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/finding-an-ami.html";
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";
const EC2_INSTANCE_PROPERTIES_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-instance.html#aws-properties-ec2-instance-properties";

// Dataset(s)
const INSTANCE_TYPES_JSON_STR: &str = include_str!("data/aws_instance_types.json");

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`AWS EC2`](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Welcome.html).
    pub async fn migrate_aws_ec2_instance(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let name = "EC2 Instance";
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

        // Create Domain Props

        // Prop: /root/domain/ImageId
        let image_id_prop = self
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

        self.create_validation(
            ctx,
            ValidationKind::Builtin(Validation::StringHasPrefix {
                value: None,
                expected: "ami-".to_string(),
            }),
            *image_id_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        let expected_instance_types: Vec<String> = serde_json::from_str(INSTANCE_TYPES_JSON_STR)?;
        let instance_types_option_list: Vec<SelectWidgetOption> = expected_instance_types
            .iter()
            .map(|instance_type| SelectWidgetOption {
                label: instance_type.to_string(),
                value: instance_type.to_string(),
            })
            .collect();
        let instance_types_option_list_json = serde_json::to_value(instance_types_option_list)?;

        // Prop: /root/domain/InstanceType
        let instance_type_prop = self
            .create_prop(
                ctx,
                "InstanceType",
                PropKind::String,
                Some((WidgetKind::ComboBox, Some(instance_types_option_list_json))),
                Some(root_prop.domain_prop_id),
                Some(EC2_INSTANCE_TYPES_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        self.create_validation(
            ctx,
            ValidationKind::Builtin(Validation::StringInStringArray {
                value: None,
                expected: expected_instance_types,
                display_expected: false,
            }),
            *instance_type_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        // Prop: /root/domain/KeyName
        let key_name_prop = self
            .create_prop(
                ctx,
                "KeyName",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_INSTANCE_PROPERTIES_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/domain/SecurityGroupIds
        let security_groups_prop = self
            .create_prop(
                ctx,
                "SecurityGroupIds",
                PropKind::Array,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_INSTANCE_PROPERTIES_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

        // Prop: /root/domain/SecurityGroupIds/SecurityGroupId
        let security_group_id_prop = self
            .create_prop(
                ctx,
                "Security Group ID",
                PropKind::String,
                None,
                Some(*security_groups_prop.id()),
                Some(EC2_INSTANCE_PROPERTIES_DOCS_URL.to_string()),
                schema_variant_id,
            )
            .await?;

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

        // Prop: /root/domain/UserData
        let user_data_prop = self
            .create_prop(
                ctx,
                "UserData",
                PropKind::String,
                Some((WidgetKind::TextArea, None)),
                Some(root_prop.domain_prop_id),
                Some(EC2_INSTANCE_PROPERTIES_DOCS_URL.to_string()),
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

        // Create Resource Props

        // Prop: /root/domain/AmiLaunchIndex
        let _ami_launch_index_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AmiLaunchIndex",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let mut image_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "ImageId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        image_id_resource_prop
            .set_refers_to_prop_id(ctx, Some(*image_id_prop.id()))
            .await?;
        image_id_resource_prop.set_default_diff(ctx).await?;

        let _instance_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "InstanceId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let mut instance_type_resource_prop = self
            .create_hidden_prop(
                ctx,
                "InstanceType",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        instance_type_resource_prop
            .set_refers_to_prop_id(ctx, Some(instance_type_prop.id()))
            .await?;
        instance_type_resource_prop.set_default_diff(ctx).await?;

        let mut key_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "KeyName",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        key_name_resource_prop
            .set_refers_to_prop_id(ctx, Some(*key_name_prop.id()))
            .await?;
        key_name_resource_prop.set_default_diff(ctx).await?;

        let _launch_time_resource_prop = self
            .create_hidden_prop(
                ctx,
                "LaunchTime",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let monitoring_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Monitoring",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _state_resource_prop = self
            .create_hidden_prop(
                ctx,
                "State",
                PropKind::String,
                Some(*monitoring_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let placement_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Placement",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _az_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AvailabilityZone",
                PropKind::String,
                Some(*placement_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _groupname_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupName",
                PropKind::String,
                Some(*placement_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _tenancy_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Tenancy",
                PropKind::String,
                Some(*placement_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _private_dns_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateDnsName",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _private_ip_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateIpAddress",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _public_dns_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PublicDnsName",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _public_ip_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PublicIpAddress",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let state_resource_prop = self
            .create_hidden_prop(
                ctx,
                "State",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _code_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Code",
                PropKind::String,
                Some(*state_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Name",
                PropKind::String,
                Some(*state_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _state_transition_reason_resource_prop = self
            .create_hidden_prop(
                ctx,
                "StateTransitionReason",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _subnet_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "SubnetId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _vpc_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "VpcId",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _architecture_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Architecture",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let block_device_mappings_resource_prop = self
            .create_hidden_prop(
                ctx,
                "BlockDeviceMappings",
                PropKind::Array,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let block_device_mapping_resource_prop = self
            .create_hidden_prop(
                ctx,
                "BlockDeviceMapping",
                PropKind::Object,
                Some(*block_device_mappings_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _device_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "DeviceName",
                PropKind::String,
                Some(*block_device_mapping_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        self.create_aws_ebs_volume_resource_prop_tree(
            ctx,
            *block_device_mapping_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        let _client_token_resource_prop = self
            .create_hidden_prop(
                ctx,
                "ClientToken",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _ebs_optimized_resource_prop = self
            .create_hidden_prop(
                ctx,
                "EbsOptimized",
                PropKind::Boolean,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _ena_support_resource_prop = self
            .create_hidden_prop(
                ctx,
                "EnaSupport",
                PropKind::Boolean,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _hypervisor_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Hypervisor",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let iam_instance_profile_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IamInstanceProfile",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _iam_instance_profile_arn_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Arn",
                PropKind::String,
                Some(*iam_instance_profile_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _iam_instance_profile_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Id",
                PropKind::String,
                Some(*iam_instance_profile_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let network_interfaces_resource_prop = self
            .create_hidden_prop(
                ctx,
                "NetworkInterfaces",
                PropKind::Array,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        self.create_aws_network_interfaces_resource_prop_tree(
            ctx,
            *network_interfaces_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        let _root_device_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "RootDeviceName",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _root_device_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "RootDeviceType",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let mut security_groups_resource_prop = self
            .create_hidden_prop(
                ctx,
                "SecurityGroups",
                PropKind::Array,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;
        security_groups_resource_prop
            .set_refers_to_prop_id(ctx, Some(*security_groups_prop.id()))
            .await?;

        let security_group_resource_prop = self
            .create_hidden_prop(
                ctx,
                "SecurityGroup",
                PropKind::Object,
                Some(*security_groups_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _security_groups_group_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupName",
                PropKind::String,
                Some(*security_group_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let mut security_groups_group_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupId",
                PropKind::String,
                Some(*security_group_resource_prop.id()),
                schema_variant_id,
            )
            .await?;
        security_groups_group_id_resource_prop
            .set_refers_to_prop_id(ctx, Some(*security_group_id_prop.id()))
            .await?;
        security_groups_group_id_resource_prop
            .set_default_diff(ctx)
            .await?;

        let _source_dest_check_resource_prop = self
            .create_hidden_prop(
                ctx,
                "SourceDestCheck",
                PropKind::Boolean,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        self.create_aws_tags_prop_tree(
            ctx,
            root_prop.resource_value_prop_id,
            schema_variant_id,
            Some(*tags_map_prop.id()),
            Some(*tags_map_item_prop.id()),
        )
        .await?;

        let _virtualization_type_resource_prop = self
            .create_hidden_prop(
                ctx,
                "VirtualizationType",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let cpu_options_resource_prop = self
            .create_hidden_prop(
                ctx,
                "CpuOptions",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _core_count_resource_prop = self
            .create_hidden_prop(
                ctx,
                "CoreCount",
                PropKind::String,
                Some(*cpu_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _threads_per_core_resource_prop = self
            .create_hidden_prop(
                ctx,
                "ThreadsPerCore",
                PropKind::String,
                Some(*cpu_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let capacity_reservation_specification_resource_prop = self
            .create_hidden_prop(
                ctx,
                "CapacityReservationSpecification",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _capacity_reservation_preferences_resource_prop = self
            .create_hidden_prop(
                ctx,
                "CapacityReservationPreference",
                PropKind::String,
                Some(*capacity_reservation_specification_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let hibernation_options_resource_prop = self
            .create_hidden_prop(
                ctx,
                "HibernationOptions",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _configured_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Configured",
                PropKind::Boolean,
                Some(*hibernation_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let metadata_options_resource_prop = self
            .create_hidden_prop(
                ctx,
                "MetadataOptions",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _metadata_state_resource_prop = self
            .create_hidden_prop(
                ctx,
                "State",
                PropKind::String,
                Some(*metadata_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _http_tokens_resource_prop = self
            .create_hidden_prop(
                ctx,
                "HttpTokens",
                PropKind::String,
                Some(*metadata_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _http_put_response_hop_limit_resource_prop = self
            .create_hidden_prop(
                ctx,
                "HttpPutResponseHopLimit",
                PropKind::String,
                Some(*metadata_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _http_endpoint_resource_prop = self
            .create_hidden_prop(
                ctx,
                "HttpEndpoint",
                PropKind::String,
                Some(*metadata_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _http_protocol_ipv6_resource_prop = self
            .create_hidden_prop(
                ctx,
                "HttpProtocolIpv6",
                PropKind::String,
                Some(*metadata_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _instance_metadata_tags_resource_prop = self
            .create_hidden_prop(
                ctx,
                "InstanceMetadataTags",
                PropKind::String,
                Some(*metadata_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let enclave_options_resource_prop = self
            .create_hidden_prop(
                ctx,
                "EnclaveOptions",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _enclave_options_enabled_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Enabled",
                PropKind::Boolean,
                Some(*enclave_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _platform_details_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PlatformDetails",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _usage_operation_resource_prop = self
            .create_hidden_prop(
                ctx,
                "UsageOperation",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _usage_operation_update_time_resource_prop = self
            .create_hidden_prop(
                ctx,
                "UsageOperationUpdateTime",
                PropKind::String,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let private_dns_name_options_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateDnsNameOptions",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _private_dns_name_options_resource_prop = self
            .create_hidden_prop(
                ctx,
                "HostnameType",
                PropKind::String,
                Some(*private_dns_name_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _enable_resource_name_dns_a_record_resource_prop = self
            .create_hidden_prop(
                ctx,
                "EnableResourceNameDnsARecord",
                PropKind::Boolean,
                Some(*private_dns_name_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _enable_resource_name_dns_aaaa_record_resource_prop = self
            .create_hidden_prop(
                ctx,
                "EnableResourceNameDnsAAAARecord",
                PropKind::Boolean,
                Some(*private_dns_name_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let maintenance_options_resource_prop = self
            .create_hidden_prop(
                ctx,
                "MaintenanceOptions",
                PropKind::Object,
                Some(root_prop.resource_value_prop_id),
                schema_variant_id,
            )
            .await?;

        let _auto_recovery_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AutoRecovery",
                PropKind::String,
                Some(*maintenance_options_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        // Add code generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:generateAwsEc2JSON", "domain")
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

        let (user_data_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "User Data",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await?;

        let (security_group_ids_explicit_internal_provider, _input_socket) =
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

        let (image_id_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Image ID",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await?;

        let (keyname_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Key Name",
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
                SocketArity::One,
                false,
            )
            .await?;

        // Qualifications
        let qualification_func_name = "si:qualificationEc2CanRun";
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
            serde_json::json!["instance"],
        )
        .await?;

        // Create a default item in the map. We will need this to connect
        // "/root/si/name" to the item's value.

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

        // Connect si/name to a tag in the tags list.
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

        // Connect props to providers.
        self.link_region_prop_to_explicit_internal_provider(
            ctx,
            region_prop.id(),
            identity_func_item.func_id,
            identity_func_item.func_argument_id,
            region_explicit_internal_provider.id(),
        )
        .await?;

        let image_id_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*image_id_prop.id());
        let image_id_attribute_value =
            AttributeValue::find_for_context(ctx, image_id_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    image_id_attribute_value_read_context,
                ))?;
        let mut image_id_attribute_prototype = image_id_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        image_id_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *image_id_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *image_id_explicit_internal_provider.id(),
        )
        .await?;

        let keyname_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*key_name_prop.id());
        let keyname_attribute_value =
            AttributeValue::find_for_context(ctx, keyname_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    keyname_attribute_value_read_context,
                ))?;
        let mut keyname_attribute_prototype = keyname_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        keyname_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *keyname_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *keyname_explicit_internal_provider.id(),
        )
        .await?;

        // Security Group Ids from input socket
        {
            let security_group_id_attribute_value_read_context =
                AttributeReadContext::default_with_prop(*security_groups_prop.id());
            let security_group_id_attribute_value = AttributeValue::find_for_context(
                ctx,
                security_group_id_attribute_value_read_context,
            )
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                security_group_id_attribute_value_read_context,
            ))?;
            let mut security_group_id_attribute_prototype = security_group_id_attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
            let (transformation_func_id, transformation_func_argument_id) = self
                .find_func_and_single_argument_by_names(ctx, "si:normalizeToArray", "value")
                .await?;
            security_group_id_attribute_prototype
                .set_func_id(ctx, transformation_func_id)
                .await?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *security_group_id_attribute_prototype.id(),
                transformation_func_argument_id,
                *security_group_ids_explicit_internal_provider.id(),
            )
            .await?;
        }

        // Consume from the user data explicit internal provider into the user data prop.
        let user_data_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*user_data_prop.id());
        let user_data_attribute_value =
            AttributeValue::find_for_context(ctx, user_data_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    user_data_attribute_value_read_context,
                ))?;
        let mut user_data_attribute_prototype = user_data_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        user_data_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *user_data_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *user_data_explicit_internal_provider.id(),
        )
        .await?;

        let workflow_func_name = "si:awsEc2CreateWorkflow";
        let workflow_func = Func::find_by_attr(ctx, "name", &workflow_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(workflow_func_name.to_owned()))?;
        let title = "Create EC2 Instance";
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
                name: "EC2 Instance Exists?".to_string(),
                success_description: Some("EC2 instance exists!".to_string()),
                failure_description: Some("This EC2 instance has not been created yet. Please run the fix above to create it!".to_string()),
                provider: Some("AWS".to_string()),
            },
        )
            .await?;

        self.add_deletion_confirmation_and_workflow(
            ctx,
            name,
            &schema_variant,
            Some("AWS"),
            "si:awsEc2DeleteWorkflow",
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

        let ec2_refresh_workflow_func_name = "si:awsEc2RefreshWorkflow";
        let ec2_refresh_workflow_func =
            Func::find_by_attr(ctx, "name", &ec2_refresh_workflow_func_name)
                .await?
                .pop()
                .ok_or_else(|| {
                    SchemaError::FuncNotFound(ec2_refresh_workflow_func_name.to_owned())
                })?;
        let title = "Refresh EC2 Instance's Resource";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *ec2_refresh_workflow_func.id(),
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

    async fn create_aws_network_interfaces_resource_prop_tree(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        let network_interface_resource_prop = self
            .create_hidden_prop(
                ctx,
                "NetworkInterface",
                PropKind::Object,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        self.create_aws_network_interface_association_resource_prop_tree(
            ctx,
            *network_interface_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        let attachment_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Attachment",
                PropKind::Object,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _attach_time_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AttachTime",
                PropKind::String,
                Some(*attachment_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _attachment_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AttachmentId",
                PropKind::String,
                Some(*attachment_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _delete_on_termination_resource_prop = self
            .create_hidden_prop(
                ctx,
                "DeleteOnTermination",
                PropKind::Boolean,
                Some(*attachment_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _device_index_resource_prop = self
            .create_hidden_prop(
                ctx,
                "DeviceIndex",
                PropKind::String,
                Some(*attachment_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _status_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Status",
                PropKind::String,
                Some(*attachment_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _network_card_index_resource_prop = self
            .create_hidden_prop(
                ctx,
                "NetworkCardIndex",
                PropKind::String,
                Some(*attachment_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _description_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Description",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let groups_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Groups",
                PropKind::Array,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let group_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Group",
                PropKind::Object,
                Some(*groups_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _group_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupName",
                PropKind::String,
                Some(*group_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _group_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "GroupId",
                PropKind::String,
                Some(*group_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let ipv6_addresses_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Ipv6Addresses",
                PropKind::Array,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let ipv6_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Ipv6Address",
                PropKind::Object,
                Some(*ipv6_addresses_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _ipv6_address_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Ipv6Address",
                PropKind::String,
                Some(*ipv6_address_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _mac_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "MacAddress",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _network_interface_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "NetworkInterfaceId",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _owner_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "OwnerId",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _private_dns_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateDnsName",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _private_ip_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateIpAddress",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _source_dest_check_resource_prop = self
            .create_hidden_prop(
                ctx,
                "SourceDestCheck",
                PropKind::Boolean,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _status_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Status",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _subnet_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "SubnetId",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _vpc_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "VpcId",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _interface_type_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "InterfaceType",
                PropKind::String,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let private_ip_addresses_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateIpAddresses",
                PropKind::Array,
                Some(*network_interface_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let p_ip_addresses_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateIpAddress",
                PropKind::Object,
                Some(*private_ip_addresses_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let p_ip_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateIpAddress",
                PropKind::Object,
                Some(*p_ip_addresses_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        self.create_aws_network_interface_association_resource_prop_tree(
            ctx,
            *p_ip_address_resource_prop.id(),
            schema_variant_id,
        )
        .await?;

        let _primary_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Primary",
                PropKind::Boolean,
                Some(*p_ip_address_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _p_private_dns_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateDnsName",
                PropKind::String,
                Some(*p_ip_address_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _p_private_ip_address_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PrivateIpAddress",
                PropKind::String,
                Some(*p_ip_address_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        Ok(())
    }

    async fn create_aws_network_interface_association_resource_prop_tree(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        let association_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Association",
                PropKind::Object,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let _ip_owner_resource_prop = self
            .create_hidden_prop(
                ctx,
                "IpOwnerId",
                PropKind::String,
                Some(*association_resource_prop.id()),
                schema_variant_id,
            )
            .await?;
        let _public_dns_name_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PublicDnsName",
                PropKind::String,
                Some(*association_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _public_ip_resource_prop = self
            .create_hidden_prop(
                ctx,
                "PublicIp",
                PropKind::String,
                Some(*association_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        Ok(())
    }

    async fn create_aws_ebs_volume_resource_prop_tree(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        let ebs_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Ebs",
                PropKind::Object,
                Some(prop_id),
                schema_variant_id,
            )
            .await?;

        let _attach_time_resource_prop = self
            .create_hidden_prop(
                ctx,
                "AttachTime",
                PropKind::String,
                Some(*ebs_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _delete_on_termination_resource_prop = self
            .create_hidden_prop(
                ctx,
                "DeleteOnTermination",
                PropKind::Boolean,
                Some(*ebs_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _status_resource_prop = self
            .create_hidden_prop(
                ctx,
                "Status",
                PropKind::String,
                Some(*ebs_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        let _volume_id_resource_prop = self
            .create_hidden_prop(
                ctx,
                "VolumeId",
                PropKind::String,
                Some(*ebs_resource_prop.id()),
                schema_variant_id,
            )
            .await?;

        Ok(())
    }
}
