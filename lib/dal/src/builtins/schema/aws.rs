use serde::{Deserialize, Serialize};

use crate::builtins::schema::{BuiltinSchemaHelpers, SelectWidgetOption};
use crate::builtins::BuiltinsError;
use crate::code_generation_prototype::CodeGenerationPrototypeContext;
use crate::edit_field::widget::WidgetKind;
use crate::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use crate::func::backend::validation::validate_string::FuncBackendValidateStringValueArgs;
use crate::func::backend::validation::validate_string_array::FuncBackendValidateStringArrayValueArgs;
use crate::qualification_prototype::QualificationPrototypeContext;
use crate::socket::{SocketArity, SocketEdgeKind, SocketKind};
use crate::validation_prototype::ValidationPrototypeContext;
use crate::{
    schema::{SchemaUiMenu, SchemaVariant},
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    BuiltinsResult, CodeGenerationPrototype, CodeLanguage, DalContext, DiagramKind,
    ExternalProvider, Func, FuncError, InternalProvider, PropKind, QualificationPrototype,
    SchemaError, SchemaKind, Socket, StandardModel, ValidationPrototype,
};

// Reference: https://aws.amazon.com/trademark-guidelines/
const AWS_NODE_COLOR: i64 = 0xFF9900;

// Documentation URLs
const AMI_DOCS_URL: &str =
    "https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html";
const EC2_DOCS_URL: &str = "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Welcome.html";
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";
const EC2_INSTANCE_TYPES_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instance-types.html";
const REGION_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html";
const KEY_PAIR_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-keypair.html";
const INGRESS_EGRESS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html";
const SECURITY_GROUP_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-security-groups.html";

// Datasets
const REGIONS: &str = include_str!("data/aws_regions.json");
const INSTANCE_TYPES: &str = include_str!("data/aws_instance_types.json");

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    ami(ctx).await?;
    ec2(ctx).await?;
    region(ctx).await?;
    keypair(ctx).await?;
    ingress(ctx).await?;
    egress(ctx).await?;
    security_group(ctx).await?;
    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AwsRegion {
    pub(crate) region_name: String,
    pub(crate) region_value: String,
}

/// A [`Schema`](crate::Schema) migration for [`AWS AMI`](https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html).
async fn ami(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "aws_ami".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "ami", "aws", &diagram_kind).await?;
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

    let mut validation_context = ValidationPrototypeContext::new();
    validation_context.set_prop_id(*image_id_prop.id());
    validation_context.set_schema_id(*schema.id());
    validation_context.set_schema_variant_id(*schema_variant.id());
    let func_name = "si:validateString".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(FuncError::NotFoundByName(func_name))?;
    ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidateStringValueArgs::new(
            None,
            "ami-".to_string(),
            true,
        ))?,
        validation_context,
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
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
        "image_id",
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
            "region",
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
    let name = "aws_ec2".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "ec2", "aws", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop and validation creation
    let mut validation_context = ValidationPrototypeContext::new();
    validation_context.set_schema_id(*schema.id());
    validation_context.set_schema_variant_id(*schema_variant.id());

    let image_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ImageId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;
    validation_context.set_prop_id(*image_id_prop.id());
    let func_name = "si:validateString".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(FuncError::NotFoundByName(func_name))?;
    ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidateStringValueArgs::new(
            None,
            "ami-".to_string(),
            true,
        ))?,
        validation_context.clone(),
    )
    .await?;

    let instance_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "InstanceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_INSTANCE_TYPES_URL.to_string()),
    )
    .await?;
    validation_context.set_prop_id(*instance_type_prop.id());
    let func_name = "si:validateStringInStringArray".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(FuncError::NotFoundByName(func_name))?;
    let expected: Vec<String> = serde_json::from_str(INSTANCE_TYPES)?;
    ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidateStringArrayValueArgs::new(
            None, expected, false,
        ))?,
        validation_context,
    )
    .await?;

    let key_name_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "KeyName",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let security_groups_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "SecurityGroupIds",
        PropKind::Array,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let _security_group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "SecurityGroupId",
        PropKind::String,
        None,
        Some(*security_groups_prop.id()),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

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
    let _tags_map_item_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let _user_data_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "UserData",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
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

    // TODO(nick): add the ability to use butane and ami as an inputs.
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    let (_butane_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "user_data",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (image_id_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "image_id",
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
            "key_name",
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
            "region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?; // TODO(wendy) - Can an EC2 instance have multiple regions? Idk!
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

    // let tags_map_attribute_read_context = AttributeReadContext {
    //     prop_id: Some(*tags_map_prop.id()),
    //     ..base_attribute_read_context
    // };
    // let tags_map_attribute_value =
    //     AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
    //         .await?
    //         .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
    //             tags_map_attribute_read_context,
    //         ))?;
    // let tags_map_item_attribute_context =
    //     AttributeContextBuilder::from(base_attribute_read_context)
    //         .set_prop_id(*tags_map_item_prop.id())
    //         .to_context()?;
    // let name_tags_item_attribute_value_id = AttributeValue::insert_for_context(
    //     ctx,
    //     tags_map_item_attribute_context,
    //     *tags_map_attribute_value.id(),
    //     None,
    //     Some("Name".to_string()),
    // )
    // .await?;

    // Connect props to providers.

    // Note(victor): The code below connects si/name to a tag in the tags list.
    // It's commented out because it breaks some tests

    // let si_name_prop =
    //     BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    // let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
    //     .await?
    //     .ok_or_else(|| {
    //         BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
    //     })?;
    // let name_tags_item_attribute_value =
    //     AttributeValue::get_by_id(ctx, &name_tags_item_attribute_value_id)
    //         .await?
    //         .ok_or(BuiltinsError::AttributeValueNotFound(
    //             name_tags_item_attribute_value_id,
    //         ))?;
    // let mut name_tags_item_attribute_prototype = name_tags_item_attribute_value
    //     .attribute_prototype(ctx)
    //     .await?
    //     .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    // name_tags_item_attribute_prototype
    //     .set_func_id(ctx, identity_func_id)
    //     .await?;
    // AttributePrototypeArgument::new_for_intra_component(
    //     ctx,
    //     *name_tags_item_attribute_prototype.id(),
    //     "identity",
    //     *si_name_internal_provider.id(),
    // )
    // .await?;

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

    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS Region`](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html).
async fn region(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "aws_region".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "region", "aws", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let regions: Vec<AwsRegion> = serde_json::from_str(REGIONS)?;
    let widget_options = regions
        .iter()
        .map(SelectWidgetOption::from)
        .collect::<Vec<SelectWidgetOption>>();
    let serialized_widget_options = serde_json::to_value(widget_options)?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        Some((WidgetKind::Select, serialized_widget_options)),
        Some(root_prop.domain_prop_id),
        Some(REGION_DOCS_URL.to_string()),
    )
    .await?;

    // Validation Creation
    let mut validation_context = ValidationPrototypeContext::new();
    validation_context.set_prop_id(*region_prop.id());
    validation_context.set_schema_id(*schema.id());
    validation_context.set_schema_variant_id(*schema_variant.id());

    let func_name = "si:validateStringInStringArray".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
    let func = funcs.pop().ok_or(FuncError::NotFoundByName(func_name))?;

    let expected = regions
        .iter()
        .map(|r| r.region_value.clone())
        .collect::<Vec<String>>();
    let _region_prop_validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidateStringArrayValueArgs::new(
            None, expected, true,
        ))?,
        validation_context,
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
        "region",
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
    let name = "key_pair".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "key pair", "aws", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let key_name_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "KeyName",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
    )
    .await?;

    let _key_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "KeyType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let _tags_map_item_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
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

    // TODO(wendy) - key_name validation, must be less than 255 ASCII characters

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
        "key_name",
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
            "region",
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

    // Connect the "region" prop to the "region" explicit internal provider.
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

/// A [`Schema`](crate::Schema) migration for [`AWS Ingress`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
async fn ingress(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "Ingress".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "Ingress", "aws", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "GroupId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _protocol_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "IpProtocol",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _to_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ToPort",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _from_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "FromPort",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _cidr_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "CidrIp",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        None, // TODO: Link documentation for aws regions
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // TODO(wendy) - validations, see Linear

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

    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    // Input Socket
    let (group_id_internal_provider, mut input_socket) =
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

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

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

    CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Set Defaults
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["security-group-rule"],
    )
    .await?;

    // Bind sockets to providers
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // region from input socket
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

    // security group id from input socket
    let group_id_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*group_id_prop.id()),
        ..base_attribute_read_context
    };
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *group_id_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *group_id_internal_provider.id(),
    )
    .await?;

    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS Egress`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
async fn egress(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "Egress".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "Egress", "aws", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "GroupId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _protocol_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "IpProtocol",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _from_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "FromPort",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _to_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ToPort",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let _cidr_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "CidrIp",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        None, // TODO: Link documentation for aws regions
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // TODO(wendy) - validations, see Linear

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

    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    // Input Socket
    let (group_id_internal_provider, mut input_socket) =
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

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

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

    CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Set Defaults
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["security-group-rule"],
    )
    .await?;

    // Bind sockets to providers
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // region from input socket
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

    // security group id from input socket
    let group_id_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*group_id_prop.id()),
        ..base_attribute_read_context
    };
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
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *group_id_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *group_id_internal_provider.id(),
    )
    .await?;

    Ok(())
}

async fn security_group(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "Security Group".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;

    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;

    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    SchemaUiMenu::new(ctx, "Security Group", "aws", &diagram_kind)
        .await?
        .set_schema(ctx, schema.id())
        .await?;

    // Prop Creation
    let security_group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "SecurityGroupId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()), // TODO: Link documentation for security groups
    )
    .await?;

    BuiltinSchemaHelpers::create_prop(
        ctx,
        "Description",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()), // TODO: Link documentation for security groups
    )
    .await?;

    BuiltinSchemaHelpers::create_prop(
        ctx,
        "GroupName",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()), // TODO: Link documentation for security groups
    )
    .await?;

    let vpc_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "VpcId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()), // TODO: Link documentation for security groups
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        None, // TODO: Link documentation for aws regions
    )
    .await?;

    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Socket Creation
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

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

    let (vpc_id_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "vpc_id",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::One,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (security_group_id_external_provider, mut output_socket) =
        ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Security Group ID",
            None,
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    output_socket.set_color(ctx, Some(0xd61e8c)).await?;

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

    CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Wrap it up!
    schema_variant.finalize(ctx).await?;

    // Set Defaults
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["security-group"],
    )
    .await?;

    // Socket Binding
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

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
        InternalProvider::get_for_prop(ctx, *security_group_id_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*security_group_id_prop.id())
            })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *security_group_id_external_provider_attribute_prototype_id,
        identity_func_identity_arg_id,
        *security_group_id_internal_provider.id(),
    )
    .await?;

    // region from input socket
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

    // vpc_id from input socket
    let vpc_id_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*vpc_id_prop.id()),
        ..base_attribute_read_context
    };
    let vpc_id_attribute_value =
        AttributeValue::find_for_context(ctx, vpc_id_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                vpc_id_attribute_value_read_context,
            ))?;
    let mut vpc_id_attribute_prototype = vpc_id_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    vpc_id_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *vpc_id_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *vpc_id_explicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}
