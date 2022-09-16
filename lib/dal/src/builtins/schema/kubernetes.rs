use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::socket::{SocketEdgeKind, SocketKind};
use crate::{
    code_generation_prototype::CodeGenerationPrototypeContext,
    func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs,
    schema::{SchemaVariant, UiMenu},
    socket::SocketArity,
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, BuiltinsError, BuiltinsResult, CodeGenerationPrototype, CodeLanguage,
    DalContext, DiagramKind, ExternalProvider, Func, InternalProvider, SchemaError, SchemaKind,
    Socket, StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};
use kubernetes_deployment::kubernetes_deployment;

mod kubernetes_deployment;
mod kubernetes_metadata;
mod kubernetes_selector;
mod kubernetes_spec;
mod kubernetes_template;

/// The default Kubernetes API version used when creating documentation URLs.
const DEFAULT_KUBERNETES_API_VERSION: &str = "1.22";

/// Provides the documentation URL prefix for a given Kubernetes documentation URL path.
pub fn doc_url(path: impl AsRef<str>) -> String {
    format!(
        "https://v{}.docs.kubernetes.io/docs/{}",
        DEFAULT_KUBERNETES_API_VERSION.replace('.', "-"),
        path.as_ref(),
    )
}

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    kubernetes_namespace(ctx).await?;
    kubernetes_deployment(ctx).await?;
    Ok(())
}

async fn kubernetes_namespace(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "kubernetes_namespace".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    let (mut variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    variant.set_color(ctx, Some(0x1ba97e)).await?;
    variant.set_link(ctx, Some("https://v1-22.docs.kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/".to_owned())).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*variant.id());

    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let mut ui_menu = UiMenu::new(ctx, &diagram_kind).await?;
    ui_menu.set_name(ctx, Some("namespace")).await?;
    ui_menu
        .set_category(ctx, Some("kubernetes".to_owned()))
        .await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let metadata_prop =
        kubernetes_metadata::create_metadata_prop(ctx, true, root_prop.domain_prop_id).await?;

    // Code Generation Prototype
    let code_generation_func_name = "si:generateYAML".to_owned();
    let mut code_generation_funcs =
        Func::find_by_attr(ctx, "name", &code_generation_func_name).await?;
    let code_generation_func = code_generation_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;
    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Yaml,
        code_generation_prototype_context,
    )
    .await?;

    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    let (external_provider, mut output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *variant.id(),
        "kubernetes_namespace",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        DiagramKind::Configuration,
    )
    .await?;
    output_socket.set_color(ctx, Some(0x85c9a3)).await?;

    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    variant.add_socket(ctx, system_socket.id()).await?;

    variant.finalize(ctx).await?;

    // Connect the "/root/si/name" field to the "/root/domain/metadata/name" field.
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };
    let metadata_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *metadata_prop.id(), "name").await?;
    let metadata_name_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*metadata_name_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await?
    .ok_or(AttributeValueError::Missing)?;
    let mut metadata_name_attribute_prototype = metadata_name_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;
    metadata_name_attribute_prototype
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
        *metadata_name_attribute_prototype.id(),
        "identity",
        *si_name_internal_provider.id(),
    )
    .await?;

    // Connect the "/root/domain/metadata/name" prop to the external provider.
    let external_provider_attribute_prototype_id =
        external_provider.attribute_prototype_id().ok_or_else(|| {
            BuiltinsError::MissingAttributePrototypeForExternalProvider(*external_provider.id())
        })?;
    let metadata_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, *metadata_prop.id(), "name").await?;
    let metadata_name_implicit_internal_provider =
        InternalProvider::get_for_prop(ctx, *metadata_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*metadata_name_prop.id())
            })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *external_provider_attribute_prototype_id,
        "identity",
        *metadata_name_implicit_internal_provider.id(),
    )
    .await?;

    let mut context = WorkflowPrototypeContext::new(); // workspace level
    context.schema_id = *schema.id();
    context.schema_variant_id = *variant.id();
    let title = "What Is Love";
    let func_name = "si:whatIsLoveWorkflow";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await?
        .pop()
        .ok_or_else(|| SchemaError::FuncNotFound(func_name.to_owned()))?;
    WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title).await?;

    Ok(())
}
