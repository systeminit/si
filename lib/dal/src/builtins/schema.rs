use crate::socket::{SocketEdgeKind, SocketKind};
use crate::{
    builtins::schema::kubernetes_metadata::create_metadata_prop,
    code_generation_prototype::CodeGenerationPrototypeContext,
    component::ComponentKind,
    func::{
        backend::{
            js_attribute::FuncBackendJsAttributeArgs,
            js_code_generation::FuncBackendJsCodeGenerationArgs,
        },
        binding::{FuncBinding, FuncBindingId},
        binding_return_value::FuncBindingReturnValueId,
    },
    schema::{SchemaVariant, UiMenu},
    socket::SocketArity,
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, BuiltinsError, BuiltinsResult, CodeGenerationPrototype, CodeLanguage,
    DalContext, DiagramKind, ExternalProvider, Func, FuncBackendKind, FuncBackendResponseType,
    FuncError, FuncId, InternalProvider, Prop, PropError, PropId, PropKind, Schema, SchemaError,
    SchemaKind, Socket, StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};

mod docker;
mod kubernetes;
mod kubernetes_deployment;
mod kubernetes_metadata;
mod kubernetes_selector;
mod kubernetes_spec;
mod kubernetes_template;
mod systeminit;

use self::kubernetes_deployment::kubernetes_deployment;

pub async fn migrate(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
    systeminit::migrate(ctx).await?;
    docker::migrate(ctx).await?;

    kubernetes_deployment(ctx).await?;
    kubernetes_namespace(ctx).await?;

    Ok(())
}

async fn kubernetes_namespace(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
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

    let metadata_prop = create_metadata_prop(ctx, true, root_prop.domain_prop_id).await?;

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

/// This unit struct (zero bytes) provides a singular place to index helpers for creating builtin
/// [`Schemas`](crate::Schema).
pub struct BuiltinSchemaHelpers;

impl BuiltinSchemaHelpers {
    pub async fn create_schema(
        ctx: &DalContext<'_, '_, '_>,
        schema_name: &str,
        schema_kind: &SchemaKind,
    ) -> BuiltinsResult<Option<Schema>> {
        // TODO(nick): there's one issue here. If the schema kind has changed, then this check will be
        // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
        // This should be fine since this code should likely only last as long as default schemas need to
        // be created... which is hopefully not long.... hopefully...
        let default_schema_exists = !Schema::find_by_attr(ctx, "name", &schema_name.to_string())
            .await?
            .is_empty();

        // TODO(nick): this should probably return an "AlreadyExists" error instead of "None", but
        // since the calling function would have to deal with the result similarly, this should suffice
        // for now.
        match default_schema_exists {
            true => Ok(None),
            false => {
                let schema =
                    Schema::new(ctx, schema_name, schema_kind, &ComponentKind::Standard).await?;
                Ok(Some(schema))
            }
        }
    }

    /// Creates a [`Prop`]. While a base [`AttributeReadContext`] is required for this function, it is
    /// only used when a parent [`PropId`] is provided.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_prop(
        ctx: &DalContext<'_, '_, '_>,
        prop_name: &str,
        prop_kind: PropKind,
        parent_prop_id: Option<PropId>,
    ) -> BuiltinsResult<Prop> {
        let prop = Prop::new(ctx, prop_name, prop_kind).await?;
        if let Some(parent_prop_id) = parent_prop_id {
            prop.set_parent_prop(ctx, parent_prop_id).await?;
        }
        Ok(prop)
    }

    pub async fn create_string_prop_with_default(
        ctx: &DalContext<'_, '_, '_>,
        prop_name: &str,
        default_string: String,
        parent_prop_id: Option<PropId>,
        _base_attribute_read_context: AttributeReadContext,
    ) -> BuiltinsResult<Prop> {
        let prop = Self::create_prop(ctx, prop_name, PropKind::String, parent_prop_id).await?;

        let mut func = Func::new(
            ctx,
            &format!("si:setDefaultToProp{:?}", prop.id()),
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::String,
        )
        .await
        .expect("cannot create func");
        func.set_handler(ctx, Some("defaultValue")).await?;
        func.set_code_base64(
            ctx,
            Some(base64::encode(&format!(
                "function defaultValue(component) {{ return \"{default_string}\"; }}"
            ))),
        )
        .await?;

        let (func_binding, func_binding_return_value) = FuncBinding::find_or_create_and_execute(
            ctx,
            // The default run doesn't have useful information, but it's just a reference for future reruns
            serde_json::to_value(FuncBackendJsAttributeArgs {
                component: veritech::ResolverFunctionComponent {
                    data: veritech::ComponentView {
                        properties: serde_json::json!({}),
                        system: None,
                        kind: veritech::ComponentKind::Standard,
                    },
                    parents: vec![],
                },
            })?,
            *func.id(),
        )
        .await?;

        let attribute_value_context = AttributeReadContext {
            prop_id: Some(*prop.id()),
            ..AttributeReadContext::default()
        };

        Prop::create_default_prototypes_and_values(ctx, *prop.id()).await?;

        let mut attribute_value = AttributeValue::find_for_context(ctx, attribute_value_context)
            .await?
            .ok_or(AttributeValueError::Missing)?;
        attribute_value
            .set_func_binding_id(ctx, *func_binding.id())
            .await?;
        attribute_value
            .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
            .await?;

        let mut attribute_prototype = attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        attribute_prototype.set_func_id(ctx, *func.id()).await?;

        Ok(prop)
    }

    /// Get the "si:identity" [`Func`](crate::Func) and execute (if necessary).
    pub async fn setup_identity_func(
        ctx: &DalContext<'_, '_, '_>,
    ) -> BuiltinsResult<(FuncId, FuncBindingId, FuncBindingReturnValueId)> {
        let identity_func_name = "si:identity".to_string();
        let identity_func: Func = Func::find_by_attr(ctx, "name", &identity_func_name)
            .await?
            .pop()
            .ok_or(FuncError::NotFoundByName(identity_func_name))?;
        let (identity_func_binding, identity_func_binding_return_value) =
            FuncBinding::find_or_create_and_execute(
                ctx,
                serde_json::json![{ "identity": null }],
                *identity_func.id(),
            )
            .await?;
        Ok((
            *identity_func.id(),
            *identity_func_binding.id(),
            *identity_func_binding_return_value.id(),
        ))
    }

    /// Find the child of a [`Prop`](crate::Prop) by name.
    ///
    /// _Use with caution!_
    pub async fn find_child_prop_by_name(
        ctx: &DalContext<'_, '_, '_>,
        prop_id: PropId,
        child_prop_name: &str,
    ) -> BuiltinsResult<Prop> {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or_else(|| PropError::NotFound(prop_id, *ctx.visibility()))?;
        for current in prop.child_props(ctx).await? {
            if current.name() == child_prop_name {
                return Ok(current);
            }
        }
        Err(PropError::ExpectedChildNotFound(child_prop_name.to_string()).into())
    }
}
