use crate::builtins::schema::BuiltinSchemaHelpers;

use crate::socket::{SocketArity, SocketEdgeKind, SocketKind};
use crate::{
    schema::{SchemaVariant, UiMenu},
    AttributeContext, BuiltinsResult, DalContext, DiagramKind, ExternalProvider, InternalProvider,
    PropKind, SchemaError, SchemaKind, Socket, StandardModel,
};

// Reference: https://aws.amazon.com/trademark-guidelines/
const AWS_PRIMARY_COLOR: i64 = 0xFF9900;
const AMI_DOCS_URL: &str =
    "https://docs.aws.amazon.com/imagebuilder/latest/APIReference/API_Ami.html";
const EC2_DOCS_URL: &str = "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/Welcome.html";

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    ami(ctx).await?;
    ec2(ctx).await?;
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
    schema_variant
        .set_color(ctx, Some(AWS_PRIMARY_COLOR))
        .await?;
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

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

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
    schema_variant
        .set_color(ctx, Some(AWS_PRIMARY_COLOR))
        .await?;
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

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

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

    Ok(())
}
