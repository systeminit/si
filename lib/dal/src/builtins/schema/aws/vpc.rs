use crate::action_prototype::ActionKind;
use crate::builtins::schema::aws::{AWS_NODE_COLOR, EC2_DOCS_URL, EC2_TAG_DOCS_URL};
use crate::builtins::schema::MigrationDriver;
use crate::builtins::BuiltinsError;
use crate::component::ComponentKind;
use crate::prototype_context::PrototypeContext;
use crate::socket::SocketArity;
use crate::validation::Validation;
use crate::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument,
    schema::SchemaUiMenu, ActionPrototype, ActionPrototypeContext, AttributePrototypeArgument,
    AttributeReadContext, AttributeValue, AttributeValueError, BuiltinsResult,
    ConfirmationPrototype, ConfirmationPrototypeContext, DalContext, DiagramKind, ExternalProvider,
    Func, FuncBinding, FuncError, InternalProvider, PropKind, QualificationPrototype,
    QualificationPrototypeContext, SchemaError, SchemaKind, SchemaVariant, StandardModel,
    WorkflowPrototype, WorkflowPrototypeContext,
};

// VPC documentation URLs
const INGRESS_EGRESS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html";
const SECURITY_GROUP_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-security-groups.html";
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";

// Datasets
const INGRESS_EGRESS_PROTOCOLS: &[&str; 3] = &["tcp", "udp", "icmp"];

impl MigrationDriver {
    pub async fn migrate_aws_vpc(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_ingress(ctx).await?;
        self.migrate_egress(ctx).await?;
        self.migrate_security_group(ctx).await?;
        Ok(())
    }

    /// A [`Schema`](crate::Schema) migration for [`AWS Ingress`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
    async fn migrate_ingress(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, schema_variant, root_prop, _) = match self
            .create_schema_and_variant(
                ctx,
                "Ingress",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(AWS_NODE_COLOR),
                None,
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
        let ui_menu = SchemaUiMenu::new(ctx, "Ingress", "AWS", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        // Prop Creation
        let group_id_prop = self
            .create_prop(
                ctx,
                "GroupId",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        let ip_permissions_prop = self
            .create_prop(
                ctx,
                "IpPermissions",
                PropKind::Array,
                None,
                Some(root_prop.domain_prop_id),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        let ip_permission_prop = self
            .create_prop(
                ctx,
                "IpPermission",
                PropKind::Object,
                None,
                Some(*ip_permissions_prop.id()),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        let _protocol_prop = self
            .create_prop(
                ctx,
                "IpProtocol",
                PropKind::String,
                None,
                Some(*ip_permission_prop.id()),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        // TODO(victor): Re add validations when they start working for objects inside arrays
        // let expected = INGRESS_EGRESS_PROTOCOLS
        //     .iter()
        //     .map(|p| p.to_string())
        //     .collect::<Vec<String>>();
        // self.create_validation(
        //     ctx,
        //     Validation::StringInStringArray {
        //         value: None,
        //         expected,
        //         display_expected: true,
        //     },
        //     *protocol_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        // )
        // .await?;

        let _to_port_prop = self
            .create_prop(
                ctx,
                "ToPort",
                PropKind::Integer,
                None,
                Some(*ip_permission_prop.id()),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        // TODO(victor): Re add validations when they start working for objects inside arrays
        // self.create_validation(
        //     ctx,
        //     Validation::IntegerIsBetweenTwoIntegers {
        //         value: None,
        //         lower_bound: -1,
        //         upper_bound: 65537,
        //     },
        //     *to_port_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        // )
        // .await?;
        //
        let _from_port_prop = self
            .create_prop(
                ctx,
                "FromPort",
                PropKind::Integer,
                None,
                Some(*ip_permission_prop.id()),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        // TODO(victor): Re add validations when they start working for objects inside arrays
        // self.create_validation(
        //     ctx,
        //     Validation::IntegerIsBetweenTwoIntegers {
        //         value: None,
        //         lower_bound: -1,
        //         upper_bound: 65537,
        //     },
        //     *from_port_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        // )
        // .await?;
        //
        let _cidr_prop = self
            .create_prop(
                ctx,
                "CidrIp",
                PropKind::String,
                None,
                Some(*ip_permission_prop.id()),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        // TODO(victor): Re add validations when they start working for objects inside arrays
        // self.create_validation(
        //     ctx,
        //     Validation::StringIsValidIpAddr { value: None },
        //     *cidr_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        // )
        // .await?;
        //
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
                Some(EC2_DOCS_URL.to_string()),
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
        let (group_id_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Security Group ID",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        input_socket.set_color(ctx, Some(0xd61e8c)).await?;

        // Input Socket
        let (exposed_ports_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Exposed Ports",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        input_socket.set_color(ctx, Some(0xd61e8c)).await?;

        let (region_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Region",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        input_socket.set_color(ctx, Some(0xd61e8c)).await?;

        // Code Generation
        let code_generation_func_name = "si:generateAwsIngressJSON".to_string();
        let code_generation_func = Func::find_by_attr(ctx, "name", &code_generation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(code_generation_func_name.to_owned()))?;
        let code_generation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "domain", *code_generation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        code_generation_func_name.clone(),
                        "domain".to_string(),
                    )
                })?;
        SchemaVariant::add_code_generation(
            ctx,
            *code_generation_func.id(),
            *code_generation_func_argument.id(),
            *schema_variant.id(),
        )
        .await?;

        // Qualifications
        let qual_func_name = "si:qualificationIngressCanCreate".to_string();
        let qual_func = Func::find_by_attr(ctx, "name", &qual_func_name)
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(qual_func_name))?;

        let mut qual_prototype_context = QualificationPrototypeContext::new();
        qual_prototype_context.set_schema_variant_id(*schema_variant.id());

        QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

        // Wrap it up.
        schema_variant.finalize(ctx).await?;

        // Set Defaults
        self.set_default_value_for_prop(
            ctx,
            *aws_resource_type_prop.id(),
            serde_json::json!["security-group-rule"],
        )
        .await?;

        // TODO(victor): Re add defaults values when they start working for objects inside arrays
        // self.set_default_value_for_prop(
        //     ctx,
        //     *ip_permissions_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        //     serde_json::json![[]],
        // )
        // .await?;
        // self.set_default_value_for_prop(
        //     ctx,
        //     *ip_permission_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        //     serde_json::json![{}],
        // )
        // .await?;

        // info!("pre");
        // self.set_default_value_for_prop(
        //     ctx,
        //     *protocol_prop.id(),
        //     *schema.id(),
        //     *schema_variant.id(),
        //     serde_json::json!["tcp"],
        // )
        // .await?;
        // info!("post");

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
        {
            let read_ctx = AttributeReadContext::default_with_prop(*group_id_prop.id());
            let attribute_value = AttributeValue::find_for_context(ctx, read_ctx)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(read_ctx))?;
            let mut attribute_prototype = attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
            attribute_prototype
                .set_func_id(ctx, identity_func_item.func_id)
                .await?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *attribute_prototype.id(),
                identity_func_item.func_argument_id,
                *group_id_internal_provider.id(),
            )
            .await?;
        }

        // Exposed Ports from input socket
        {
            let transformation_func_name = "si:dockerPortsToAwsIngressPorts".to_string();
            let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
                .await?
                .pop()
                .ok_or_else(|| FuncError::NotFoundByName(transformation_func_name.clone()))?;
            let arg_name = "ExposedPorts";
            let arg = FuncArgument::find_by_name_for_func(ctx, arg_name, *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        transformation_func_name.clone(),
                        arg_name.to_string(),
                    )
                })?;

            let read_ctx = AttributeReadContext::default_with_prop(*ip_permissions_prop.id());

            let attribute_value = AttributeValue::find_for_context(ctx, read_ctx)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(read_ctx))?;

            let mut attribute_prototype = attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;

            attribute_prototype
                .set_func_id(ctx, *transformation_func.id())
                .await?;

            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *attribute_prototype.id(),
                *arg.id(),
                *exposed_ports_internal_provider.id(),
            )
            .await?;
        }

        let func_name = "si:awsIngressCreateWorkflow";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let title = "Create Ingress";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype =
            WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title)
                .await?;

        let func_name = "si:resourceExistsConfirmation";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let context = ConfirmationPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let mut confirmation_prototype =
            ConfirmationPrototype::new(ctx, "Ingress Exists?", *func.id(), context).await?;
        confirmation_prototype
            .set_provider(ctx, Some("AWS".to_owned()))
            .await?;
        confirmation_prototype
            .set_success_description(ctx, Some("Ingress exists!".to_owned()))
            .await?;
        confirmation_prototype
            .set_failure_description(
                ctx,
                Some(
                    "This Ingress rule has not been created yet. Please run the fix above to create it!"
                        .to_owned(),
                ),
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

        let func_name = "si:awsIngressRefreshWorkflow";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let title = "Refresh Ingress's Resource";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype =
            WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title)
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

        Ok(())
    }

    /// A [`Schema`](crate::Schema) migration for [`AWS Egress`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
    async fn migrate_egress(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, schema_variant, root_prop, _) = match self
            .create_schema_and_variant(
                ctx,
                "Egress",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(AWS_NODE_COLOR),
                None,
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
        let ui_menu = SchemaUiMenu::new(ctx, "Egress", "AWS", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        // Prop Creation
        let group_id_prop = self
            .create_prop(
                ctx,
                "GroupId",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
            )
            .await?;

        let protocol_prop = self
            .create_prop(
                ctx,
                "IpProtocol",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
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
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
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
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
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
                Some(INGRESS_EGRESS_DOCS_URL.to_string()),
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
                Some(EC2_DOCS_URL.to_string()),
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
        let (group_id_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Security Group ID",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        input_socket.set_color(ctx, Some(0xd61e8c)).await?;

        let (region_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Region",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        input_socket.set_color(ctx, Some(0xd61e8c)).await?;

        // Code Generation
        let code_generation_func_name = "si:generateAwsEgressJSON".to_string();
        let code_generation_func = Func::find_by_attr(ctx, "name", &code_generation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(code_generation_func_name.to_owned()))?;
        let code_generation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "domain", *code_generation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        code_generation_func_name.clone(),
                        "domain".to_string(),
                    )
                })?;
        SchemaVariant::add_code_generation(
            ctx,
            *code_generation_func.id(),
            *code_generation_func_argument.id(),
            *schema_variant.id(),
        )
        .await?;

        // Qualifications
        let qual_func_name = "si:qualificationEgressCanCreate".to_string();
        let qual_func = Func::find_by_attr(ctx, "name", &qual_func_name)
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(qual_func_name))?;

        let mut qual_prototype_context = QualificationPrototypeContext::new();
        qual_prototype_context.set_schema_variant_id(*schema_variant.id());

        QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

        // Wrap it up.
        schema_variant.finalize(ctx).await?;

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

        let func_name = "si:awsEgressCreateWorkflow";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let title = "Create Egress";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype =
            WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title)
                .await?;

        let func_name = "si:resourceExistsConfirmation";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let context = ConfirmationPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let mut confirmation_prototype =
            ConfirmationPrototype::new(ctx, "Egress Exists?", *func.id(), context).await?;
        confirmation_prototype
            .set_provider(ctx, Some("AWS".to_owned()))
            .await?;
        confirmation_prototype
            .set_success_description(ctx, Some("Egress exists!".to_owned()))
            .await?;
        confirmation_prototype
            .set_failure_description(
                ctx,
                Some(
                    "This Egress rule has not been created yet. Please run the fix above to create it!"
                        .to_owned(),
                ),
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

        let func_name = "si:awsEgressRefreshWorkflow";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let title = "Refresh Egress's Resource";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype =
            WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title)
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

        Ok(())
    }

    /// A [`Schema`](crate::Schema) migration for [`AWS Security Group`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
    async fn migrate_security_group(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, schema_variant, root_prop, _) = match self
            .create_schema_and_variant(
                ctx,
                "Security Group",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(AWS_NODE_COLOR),
                None,
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
        SchemaUiMenu::new(ctx, "Security Group", "AWS", &diagram_kind)
            .await?
            .set_schema(ctx, schema.id())
            .await?;

        // Prop Creation
        self.create_prop(
            ctx,
            "Description",
            PropKind::String,
            None,
            Some(root_prop.domain_prop_id),
            Some(SECURITY_GROUP_DOCS_URL.to_string()),
        )
        .await?;

        let group_name_prop = self
            .create_prop(
                ctx,
                "GroupName",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(SECURITY_GROUP_DOCS_URL.to_string()),
            )
            .await?;

        let _vpc_id_prop = self
            .create_prop(
                ctx,
                "VpcId",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(SECURITY_GROUP_DOCS_URL.to_string()),
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

        let aws_resource_type_prop = self
            .create_prop(
                ctx,
                "awsResourceType",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_DOCS_URL.to_string()),
            )
            .await?;

        // Socket Creation
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;
        let (region_explicit_internal_provider, mut input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Region",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        input_socket.set_color(ctx, Some(0xd61e8c)).await?;

        let func_name = "si:awsSecurityGroupIdFromResource";
        let aws_security_group_id_from_resource_func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let (func_binding, func_binding_return_value, _) = FuncBinding::find_or_create_and_execute(
            ctx,
            serde_json::json!({}),
            *aws_security_group_id_from_resource_func.id(),
        )
        .await?;

        let (security_group_id_external_provider, mut output_socket) =
            ExternalProvider::new_with_socket(
                ctx,
                *schema.id(),
                *schema_variant.id(),
                "Security Group ID",
                None,
                *aws_security_group_id_from_resource_func.id(),
                *func_binding.id(),
                *func_binding_return_value.id(),
                SocketArity::Many,
                DiagramKind::Configuration,
            )
            .await?;
        output_socket.set_color(ctx, Some(0xd61e8c)).await?;

        // Code Generation
        let code_generation_func_name = "si:generateAwsSecurityGroupJSON".to_string();
        let code_generation_func = Func::find_by_attr(ctx, "name", &code_generation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(code_generation_func_name.to_owned()))?;
        let code_generation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "domain", *code_generation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        code_generation_func_name.clone(),
                        "domain".to_string(),
                    )
                })?;
        SchemaVariant::add_code_generation(
            ctx,
            *code_generation_func.id(),
            *code_generation_func_argument.id(),
            *schema_variant.id(),
        )
        .await?;

        // Qualifications
        let qual_func_name = "si:qualificationSecurityGroupCanCreate".to_string();
        let qual_func = Func::find_by_attr(ctx, "name", &qual_func_name)
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(qual_func_name))?;

        let mut qual_prototype_context = QualificationPrototypeContext::new();
        qual_prototype_context.set_schema_variant_id(*schema_variant.id());

        QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

        // Wrap it up!
        schema_variant.finalize(ctx).await?;

        // Set Defaults
        self.set_default_value_for_prop(
            ctx,
            *aws_resource_type_prop.id(),
            serde_json::json!["security-group"],
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

        let func_argument = FuncArgument::find_by_name_for_func(
            ctx,
            "resource",
            *aws_security_group_id_from_resource_func.id(),
        )
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
        let si_name_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "name")
            .await?;
        let si_name_internal_provider = InternalProvider::find_for_prop(ctx, *si_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
            })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *group_name_attribute_proto.id(),
            identity_func_item.func_argument_id,
            *si_name_internal_provider.id(),
        )
        .await?;

        let func_name = "si:awsSecurityGroupCreateWorkflow";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let title = "Create Security Group";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype =
            WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title)
                .await?;

        let func_name = "si:resourceExistsConfirmation";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let context = ConfirmationPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let mut confirmation_prototype =
            ConfirmationPrototype::new(ctx, "Security Group Exists?", *func.id(), context).await?;
        confirmation_prototype
            .set_provider(ctx, Some("AWS".to_owned()))
            .await?;
        confirmation_prototype
            .set_success_description(ctx, Some("Security Group exists!".to_owned()))
            .await?;
        confirmation_prototype.set_failure_description(ctx, Some("This Security Group has not been created yet. Please run the fix above to create it!".to_owned())).await?;

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

        let func_name = "si:awsSecurityGroupRefreshWorkflow";
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
        let title = "Refresh Security Group's Resource";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype =
            WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title)
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

        Ok(())
    }
}
