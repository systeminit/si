use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::builtins::BuiltinsError;
use crate::socket::{SocketArity, SocketEdgeKind, SocketKind};
use crate::{
    schema::{SchemaVariant, UiMenu},
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    BuiltinsResult, DalContext, DiagramKind, ExternalProvider, InternalProvider, PropKind,
    SchemaError, SchemaKind, Socket, StandardModel,
};

// Reference: https://aws.amazon.com/trademark-guidelines/
const AWS_NODE_COLOR: i64 = 0xFF9900;

// Documentation URLs.
const AMI_DOCS_URL: &str =
    "https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html";
const EC2_DOCS_URL: &str = "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Welcome.html";
const REGION_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html";
const KEY_PAIR_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-keypair.html";

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    ami(ctx).await?;
    ec2(ctx).await?;
    region(ctx).await?;
    keypair(ctx).await?;
    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS AMI`](https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html).
async fn ami(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "aws_ami".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup. The variant color was taken from the coreos logo.
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
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("ami")).await?;
    ui_menu.set_category(ctx, Some("aws".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop creation
    // TODO(nick): add validation that max length is 1024 characters. This is mentioned in the
    // reference.
    let _image_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "image",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(AMI_DOCS_URL.to_string()),
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

    // TODO(nick): add ability to export image id for ec2.
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
    let (_ec2_image_id_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
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

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

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

    // Variant setup. The variant color was taken from the coreos logo.
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
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("ec2")).await?;
    ui_menu.set_category(ctx, Some("aws".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop creation
    // TODO(nick): add validation for shape (e.g. "ami-XXX").
    let _image_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "imageId",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;
    let _user_data_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "userData",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;
    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
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
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
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
    let (_ami_explicit_internal_provider, mut input_socket) =
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

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

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
        "identity",
        *region_explicit_internal_provider.id(),
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

    // Variant setup. The variant color was taken from the coreos logo.
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
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("region")).await?;
    ui_menu.set_category(ctx, Some("aws".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(REGION_DOCS_URL.to_string()),
    )
    .await?;

    // Validation Creation
    // let mut validation_context = ValidationPrototypeContext::new();
    // validation_context.set_prop_id(*region_prop.id());
    // validation_context.set_schema_id(*schema.id());
    // validation_context.set_schema_variant_id(*schema_variant.id());

    // FuncBackendValidateStringArrayValueArgs::new();

    // ValidationPrototype::new(ctx, func_id, args, validation_context).await?;

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
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
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
        "identity",
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

    // Variant setup. The variant color was taken from the coreos logo.
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
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("key pair")).await?;
    ui_menu.set_category(ctx, Some("aws".to_owned())).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let key_name_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "key name",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
    )
    .await?;

    let _key_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "key type",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
    )
    .await?;

    let _tags_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Array,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
    )
    .await?;
    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        Some(root_prop.domain_prop_id),
        Some(KEY_PAIR_DOCS_URL.to_string()),
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
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;
    let (key_name_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "key id",
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

    // Connect the "/root/domain/key id" prop to the external provider.
    // TODO(wendy) - currently this just gives the key name, how can we give the key id without making it a prop?
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
        "key name",
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
        "identity",
        *region_explicit_internal_provider.id(),
    )
    .await?;

    Ok(())
}
