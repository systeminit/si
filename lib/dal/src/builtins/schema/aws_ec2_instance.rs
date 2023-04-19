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
use crate::SchemaVariant;
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
            .create_schema_and_variant_with_name(
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
                "v1",
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        // Prop: /root/domain/ImageId
        let image_id_prop = self
            .create_prop(
                ctx,
                "ImageId",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(AMI_DOCS_URL.to_string()),
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

        // Prop: /root/domain/key_outdated
        let mut key_outdated_prop = self
            .create_prop(
                ctx,
                "key_outdated",
                PropKind::Boolean,
                None,
                Some(root_prop.domain_prop_id),
                None,
            )
            .await?;
        key_outdated_prop.set_hidden(ctx, true).await?;

        // Prop: /root/domain/KeyName
        let key_name_prop = self
            .create_prop(
                ctx,
                "KeyName",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_INSTANCE_PROPERTIES_DOCS_URL.to_string()),
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
            )
            .await?;

        // Prop: /root/domain/SecurityGroupIds/SecurityGroupId
        let _security_group_id_prop = self
            .create_prop(
                ctx,
                "Security Group ID",
                PropKind::String,
                None,
                Some(*security_groups_prop.id()),
                Some(EC2_INSTANCE_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        // Prop: /root/domain/security_groups_outdated
        let mut security_groups_outdated_prop = self
            .create_prop(
                ctx,
                "security_groups_outdated",
                PropKind::Array,
                None,
                Some(root_prop.domain_prop_id),
                None,
            )
            .await?;
        security_groups_outdated_prop.set_hidden(ctx, true).await?;

        // Prop: /root/domain/security_groups_outdated/security_group_outdated
        let mut security_group_outdated_prop = self
            .create_prop(
                ctx,
                "security_group_oudated",
                PropKind::Boolean,
                None,
                Some(*security_groups_outdated_prop.id()),
                None,
            )
            .await?;
        security_group_outdated_prop.set_hidden(ctx, true).await?;

        // Prop: /root/domain/tags
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

        // Prop: /root/domain/tags/tag
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

        // Prop: /root/domain/UserData
        let user_data_prop = self
            .create_prop(
                ctx,
                "UserData",
                PropKind::String,
                Some((WidgetKind::TextArea, None)),
                Some(root_prop.domain_prop_id),
                Some(EC2_INSTANCE_PROPERTIES_DOCS_URL.to_string()),
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

        let (security_group_ids_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Security Group ID2",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await?;
        input_socket
            .set_human_name(ctx, Some("Security Group ID"))
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

        let (keyname_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Key Name2",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await?;
        input_socket.set_human_name(ctx, Some("Key Name")).await?;

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

        // Create and set the func to take off a string field.
        let transformation_func_name = "si:awsKeyPairKeyNameFromSocket";
        let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(transformation_func_name.to_owned()))?;
        let transformation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "key", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        transformation_func_name.to_owned(),
                        "key".to_string(),
                    )
                })?;

        let key_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*key_name_prop.id());
        let key_attribute_value =
            AttributeValue::find_for_context(ctx, key_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    key_attribute_value_read_context,
                ))?;
        let mut key_attribute_prototype = key_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        key_attribute_prototype
            .set_func_id(ctx, transformation_func.id())
            .await?;

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *key_attribute_prototype.id(),
            *transformation_func_argument.id(),
            *keyname_explicit_internal_provider.id(),
        )
        .await?;

        let transformation_func_name = "si:getOutdated";
        let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(transformation_func_name.to_owned()))?;
        let transformation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "value", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        transformation_func_name.to_owned(),
                        "value".to_string(),
                    )
                })?;

        let key_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*key_outdated_prop.id());
        let key_attribute_value =
            AttributeValue::find_for_context(ctx, key_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    key_attribute_value_read_context,
                ))?;
        let mut key_attribute_prototype = key_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        key_attribute_prototype
            .set_func_id(ctx, transformation_func.id())
            .await?;

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *key_attribute_prototype.id(),
            *transformation_func_argument.id(),
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
                .find_func_and_single_argument_by_names(
                    ctx,
                    "si:awsEc2InstanceNormalizeSecurityGroupIdFromSocket",
                    "value",
                )
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

        // Security Group Ids Outdated from input socket
        {
            let security_group_outdated_attribute_value_read_context =
                AttributeReadContext::default_with_prop(*security_groups_outdated_prop.id());
            let security_group_outdated_attribute_value = AttributeValue::find_for_context(
                ctx,
                security_group_outdated_attribute_value_read_context,
            )
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                security_group_outdated_attribute_value_read_context,
            ))?;
            let mut security_group_outdated_attribute_prototype =
                security_group_outdated_attribute_value
                    .attribute_prototype(ctx)
                    .await?
                    .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
            let (transformation_func_id, transformation_func_argument_id) = self
                .find_func_and_single_argument_by_names(ctx, "si:normalizeOutdated", "value")
                .await?;
            security_group_outdated_attribute_prototype
                .set_func_id(ctx, transformation_func_id)
                .await?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *security_group_outdated_attribute_prototype.id(),
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

        // Add update confirmation
        self.add_update_confirmation(ctx, name, &schema_variant, Some("AWS"))
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
}
