use serde::{Deserialize, Serialize};

use crate::builtins::schema::{BuiltinSchemaHelpers, SelectWidgetOption};
use crate::builtins::BuiltinsError;
use crate::code_generation_prototype::CodeGenerationPrototypeContext;
use crate::component::ComponentKind;
use crate::edit_field::widget::WidgetKind;
use crate::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use crate::prototype_context::PrototypeContext;
use crate::qualification_prototype::QualificationPrototypeContext;
use crate::socket::{SocketArity, SocketEdgeKind, SocketKind};
use crate::validation::Validation;
use crate::AttributeValueError;
use crate::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument,
    schema::SchemaUiMenu, AttributeContext, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, BuiltinsResult, CodeGenerationPrototype, CodeLanguage, DalContext, DiagramKind,
    ExternalProvider, Func, InternalProvider, PropKind, QualificationPrototype, SchemaError,
    SchemaKind, Socket, StandardModel,
};

mod vpc;

// Reference: https://aws.amazon.com/trademark-guidelines/
const AWS_NODE_COLOR: i64 = 0xFF9900;

// Documentation URLs
const AMI_DOCS_URL: &str =
    "https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html";
const EC2_DOCS_URL: &str = "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Welcome.html";
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";
const EC2_INSTANCE_TYPES_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instance-types.html";
const KEY_PAIR_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-keypair.html";

// Datasets
const REGIONS_JSON_STR: &str = include_str!("data/aws_regions.json");
const INSTANCE_TYPES_JSON_STR: &str = include_str!("data/aws_instance_types.json");

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    ami(ctx).await?;
    ec2(ctx).await?;
    region(ctx).await?;
    keypair(ctx).await?;
    vpc::migrate(ctx).await?;
    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AwsRegion {
    pub(crate) name: String,
    pub(crate) code: String,
}

impl From<&AwsRegion> for SelectWidgetOption {
    fn from(region: &AwsRegion) -> Self {
        Self {
            label: format!("{} - {}", region.code, region.name),
            value: region.code.clone(),
        }
    }
}

/// A [`Schema`](crate::Schema) migration for [`AWS AMI`](https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html).
async fn ami(ctx: &DalContext) -> BuiltinsResult<()> {
    let (schema, schema_variant, root_prop) = match BuiltinSchemaHelpers::create_schema_and_variant(
        ctx,
        "AMI",
        SchemaKind::Configuration,
        ComponentKind::Standard,
        Some(AWS_NODE_COLOR),
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
    let ui_menu = SchemaUiMenu::new(ctx, "AMI", "AWS", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop and validation creation
    let image_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ImageId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(AMI_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringHasPrefix {
            value: None,
            expected: "ami-".to_string(),
        },
        *image_id_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(AWS_REGIONS_DOCS_URL.to_string()),
    )
    .await?;

    // Code Generation
    let code_generation_func_name = "si:generateAwsJSON".to_owned();
    let code_generation_func =
        Func::find_by_attr(ctx, "name", &code_generation_func_name.to_owned())
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;

    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

    let _prototype = CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Sockets
    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    let (image_id_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "Image ID",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;
    output_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Qualifications
    let qual_func_name = "si:qualificationAmiExists".to_string();

    let qual_func = Func::find_by_attr(ctx, "name", &qual_func_name)
        .await?
        .pop()
        .ok_or(SchemaError::FuncNotFound(qual_func_name))?;

    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*schema_variant.id());

    QualificationPrototype::new(ctx, *qual_func.id(), qual_prototype_context).await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Connect the props to providers.
    let external_provider_attribute_prototype_id = image_id_external_provider
        .attribute_prototype_id()
        .ok_or_else(|| {
            BuiltinsError::MissingAttributePrototypeForExternalProvider(
                *image_id_external_provider.id(),
            )
        })?;
    let image_id_implicit_internal_provider =
        InternalProvider::get_for_prop(ctx, *image_id_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*image_id_prop.id())
            })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *external_provider_attribute_prototype_id,
        identity_func_identity_arg_id,
        *image_id_implicit_internal_provider.id(),
    )
    .await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };
    let region_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*region_prop.id()),
        ..base_attribute_read_context
    };
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *region_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *region_explicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS EC2`](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Welcome.html).
async fn ec2(ctx: &DalContext) -> BuiltinsResult<()> {
    let (schema, schema_variant, root_prop) = match BuiltinSchemaHelpers::create_schema_and_variant(
        ctx,
        "EC2 Instance",
        SchemaKind::Configuration,
        ComponentKind::Standard,
        Some(AWS_NODE_COLOR),
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
    let ui_menu = SchemaUiMenu::new(ctx, "EC2 Instance", "AWS", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop: /root/domain/ImageId
    let image_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ImageId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringHasPrefix {
            value: None,
            expected: "ami-".to_string(),
        },
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
    let instance_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "InstanceType",
        PropKind::String,
        Some((WidgetKind::Select, instance_types_option_list_json)),
        Some(root_prop.domain_prop_id),
        Some(EC2_INSTANCE_TYPES_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringInStringArray {
            value: None,
            expected: expected_instance_types,
            display_expected: false,
        },
        *instance_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    // Prop: /root/domain/KeyName
    let key_name_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "KeyName",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/SecurityGroupIds
    let security_groups_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "SecurityGroupIds",
        PropKind::Array,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/SecurityGroupIds/SecurityGroupId
    let _security_group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "Security Group ID",
        PropKind::String,
        None,
        Some(*security_groups_prop.id()),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/tags
    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // TODO(victor): Make one item of the list have key `Name` and value equal to /root/si/name
    // Prop: /root/domain/tags/tag
    let tags_map_item_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/UserData
    let user_data_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "UserData",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/awsResourceType
    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/region
    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(AWS_REGIONS_DOCS_URL.to_string()),
    )
    .await?;

    // Code Generation Prototype
    let code_generation_func_name = "si:generateAwsJSON".to_owned();
    let code_generation_func =
        Func::find_by_attr(ctx, "name", &code_generation_func_name.to_owned())
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;

    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

    let _prototype = CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Sockets
    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    let (image_id_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Image ID",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (security_group_ids_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Security Group ID",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (keyname_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Key Name",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?; // TODO(wendy) - Can an EC2 instance have multiple regions? Idk!
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (user_data_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "User Data",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Qualifications
    let qual_func_name = "si:qualificationEc2CanRun".to_string();

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
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["instance"],
    )
    .await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // Note(victor): The code below is commented out because it breaks some tests. We should come back to this someday.
    // Create a default item in the map. We will need this to connect
    // "/root/si/name" to the item's value.

    let tags_map_attribute_read_context = AttributeReadContext {
        prop_id: Some(*tags_map_prop.id()),
        ..base_attribute_read_context
    };
    let tags_map_attribute_value =
        AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                tags_map_attribute_read_context,
            ))?;
    let tags_map_item_attribute_context =
        AttributeContextBuilder::from(base_attribute_read_context)
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

    // Note(victor): The code below connects si/name to a tag in the tags list.
    // It's commented out because it breaks some tests

    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    let identity_arg = FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
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
    let region_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*region_prop.id()),
        ..base_attribute_read_context
    };
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *region_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *region_explicit_internal_provider.id(),
    )
    .await?;

    let image_id_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*image_id_prop.id()),
        ..base_attribute_read_context
    };
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *image_id_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *image_id_explicit_internal_provider.id(),
    )
    .await?;

    let keyname_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*key_name_prop.id()),
        ..base_attribute_read_context
    };
    let keyname_attribute_value =
        AttributeValue::find_for_context(ctx, keyname_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                keyname_attribute_value_read_context,
            ))?;
    let mut keyname_attribute_prototype =
        keyname_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    keyname_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *keyname_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *keyname_explicit_internal_provider.id(),
    )
    .await?;

    let security_group_id_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*security_groups_prop.id()),
        ..base_attribute_read_context
    };
    let security_group_id_attribute_value =
        AttributeValue::find_for_context(ctx, security_group_id_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                security_group_id_attribute_value_read_context,
            ))?;
    let mut security_group_id_attribute_prototype = security_group_id_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    security_group_id_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *security_group_id_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *security_group_ids_explicit_internal_provider.id(),
    )
    .await?;

    let user_data_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*user_data_prop.id()),
        ..base_attribute_read_context
    };
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *user_data_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *user_data_explicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS Region`](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html).
async fn region(ctx: &DalContext) -> BuiltinsResult<()> {
    let (schema, schema_variant, root_prop) = match BuiltinSchemaHelpers::create_schema_and_variant(
        ctx,
        "Region",
        SchemaKind::Configuration,
        ComponentKind::Standard,
        Some(AWS_NODE_COLOR),
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
    let ui_menu = SchemaUiMenu::new(ctx, "Region", "AWS", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let regions_json: Vec<AwsRegion> = serde_json::from_str(REGIONS_JSON_STR)?;
    let regions_dropdown_options = regions_json
        .iter()
        .map(SelectWidgetOption::from)
        .collect::<Vec<SelectWidgetOption>>();
    let regions_dropdown_options_json = serde_json::to_value(regions_dropdown_options)?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        Some((WidgetKind::Select, regions_dropdown_options_json)),
        Some(root_prop.domain_prop_id),
        Some(AWS_REGIONS_DOCS_URL.to_string()),
    )
    .await?;

    // Validation Creation
    let expected = regions_json
        .iter()
        .map(|r| r.code.clone())
        .collect::<Vec<String>>();
    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringInStringArray {
            value: None,
            expected,
            display_expected: true,
        },
        *region_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    // System Socket
    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    // Output Socket
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
    let (region_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "Region",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;
    output_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Connect the "/root/domain/region" prop to the external provider.
    let external_provider_attribute_prototype_id = region_external_provider
        .attribute_prototype_id()
        .ok_or_else(|| {
            BuiltinsError::MissingAttributePrototypeForExternalProvider(
                *region_external_provider.id(),
            )
        })?;
    let region_implicit_internal_provider = InternalProvider::get_for_prop(ctx, *region_prop.id())
        .await?
        .ok_or_else(|| BuiltinsError::ImplicitInternalProviderNotFoundForProp(*region_prop.id()))?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *external_provider_attribute_prototype_id,
        identity_func_identity_arg_id,
        *region_implicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS Key Pair`](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-keypair.html).
async fn keypair(ctx: &DalContext) -> BuiltinsResult<()> {
    let (schema, schema_variant, root_prop) = match BuiltinSchemaHelpers::create_schema_and_variant(
        ctx,
        "Key Pair",
        SchemaKind::Configuration,
        ComponentKind::Standard,
        Some(AWS_NODE_COLOR),
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
    let ui_menu = SchemaUiMenu::new(ctx, "Key Pair", "AWS", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop: /root/domain/KeyName
    let key_name_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "KeyName",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/KeyType
    let _key_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "KeyType",
        PropKind::String,
        Some((
            WidgetKind::Select,
            serde_json::json!([
                {
                    "label": "rsa",
                    "value": "rsa",
                },
                {
                    "label": "ed25519",
                    "value": "ed25519",
                },
            ]),
        )),
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/tags
    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/tags/tag
    let tags_map_item_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/awsResourceType
    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Prop: /root/domain/region
    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(AWS_REGIONS_DOCS_URL.to_string()),
    )
    .await?;

    // Code Generation Prototype
    let code_generation_func_name = "si:generateAwsJSON".to_owned();
    let code_generation_func =
        Func::find_by_attr(ctx, "name", &code_generation_func_name.to_owned())
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;

    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

    let _prototype = CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // System Socket
    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    // Output Socket
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
    let (key_name_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "Key Name",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;
    output_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Input Socket
    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?; // TODO(wendy) - Can a key pair have multiple regions? Idk!
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Set Defaults
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["key-pair"],
    )
    .await?;

    // Connect the "/root/domain/key id" prop to the external provider.
    let external_provider_attribute_prototype_id = key_name_external_provider
        .attribute_prototype_id()
        .ok_or_else(|| {
            BuiltinsError::MissingAttributePrototypeForExternalProvider(
                *key_name_external_provider.id(),
            )
        })?;
    let key_name_internal_provider = InternalProvider::get_for_prop(ctx, *key_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*key_name_prop.id())
        })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *external_provider_attribute_prototype_id,
        identity_func_identity_arg_id,
        *key_name_internal_provider.id(),
    )
    .await?;

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    let tags_map_attribute_read_context = AttributeReadContext {
        prop_id: Some(*tags_map_prop.id()),
        ..base_attribute_read_context
    };
    let tags_map_attribute_value =
        AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                tags_map_attribute_read_context,
            ))?;
    let tags_map_item_attribute_context =
        AttributeContextBuilder::from(base_attribute_read_context)
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

    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    let identity_arg = FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
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
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };
    let region_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*region_prop.id()),
        ..base_attribute_read_context
    };
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *region_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *region_explicit_internal_provider.id(),
    )
    .await?;

    // Connect the "/root/si/name" field to the "/root/domain/KeyName" field.
    let key_name_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*key_name_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await?
    .ok_or(AttributeValueError::Missing)?;
    let mut key_name_attribute_prototype = key_name_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;
    key_name_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
        })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *key_name_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *si_name_internal_provider.id(),
    )
    .await?;

    Ok(())
}
