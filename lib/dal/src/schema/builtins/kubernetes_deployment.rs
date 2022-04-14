use crate::code_generation_prototype::CodeGenerationPrototypeContext;
use crate::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use crate::func::backend::js_qualification::FuncBackendJsQualificationArgs;
use crate::qualification_prototype::QualificationPrototypeContext;
use crate::schema::builtins::kubernetes_metadata::create_metadata_prop;
use crate::schema::builtins::kubernetes_selector::create_selector_prop;
use crate::schema::builtins::kubernetes_template::create_template_prop;
use crate::schema::builtins::{create_prop, create_schema, create_string_prop_with_default};
use crate::schema::{SchemaResult, SchemaVariant, UiMenu};
use crate::socket::{Socket, SocketArity, SocketEdgeKind};
use crate::{
    AttributeReadContext, CodeGenerationPrototype, DalContext, Func, PropKind,
    QualificationPrototype, Schema, SchemaError, SchemaKind, SchematicKind, StandardModel,
};

pub async fn kubernetes_deployment(ctx: &DalContext<'_, '_>) -> SchemaResult<()> {
    let name = "kubernetes_deployment".to_string();
    let mut schema = match create_schema(ctx, &name, &SchemaKind::Concrete).await? {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*variant.id()))
        .await?;
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*variant.id()),
        ..AttributeReadContext::default()
    };

    {
        // TODO: add validation (si-registry ensures the value is unchanged)
        let _api_version_prop = create_string_prop_with_default(
            ctx,
            "apiVersion",
            "apps/v1".to_owned(),
            Some(root_prop.domain_prop_id),
            base_attribute_read_context,
        )
        .await?;
    }
    {
        // TODO: add validation (si-registry ensures the value is unchanged)
        let _kind_prop = create_string_prop_with_default(
            ctx,
            "kind",
            "Deployment".to_owned(),
            Some(root_prop.domain_prop_id),
            base_attribute_read_context,
        )
        .await?;
    }

    {
        let _metadata_prop = create_metadata_prop(
            ctx,
            true, // is name required, note: bool is not ideal here tho
            Some(root_prop.domain_prop_id),
            base_attribute_read_context,
        )
        .await?;
    }

    {
        let spec_prop = create_prop(
            ctx,
            "spec",
            PropKind::Object,
            Some(root_prop.domain_prop_id),
            base_attribute_read_context,
        )
        .await?;

        {
            let _replicas_prop = create_prop(
                ctx,
                "replicas",
                PropKind::Integer,
                Some(*spec_prop.id()),
                base_attribute_read_context,
            )
            .await?;
        }

        {
            let _selector_prop =
                create_selector_prop(ctx, Some(*spec_prop.id()), base_attribute_read_context)
                    .await?;
        }

        {
            let _template_prop =
                create_template_prop(ctx, Some(*spec_prop.id()), base_attribute_read_context)
                    .await?;
        }
    }

    // Qualification Prototype
    let qualification_func_name = "si:qualificationYamlKubeval".to_owned();
    let mut qualification_funcs = Func::find_by_attr(ctx, "name", &qualification_func_name).await?;
    let qualification_func = qualification_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qualification_func_name))?;
    let qualification_args = FuncBackendJsQualificationArgs::default();
    let qualification_args_json = serde_json::to_value(&qualification_args)?;
    let mut qualification_prototype_context = QualificationPrototypeContext::new();
    qualification_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = QualificationPrototype::new(
        ctx,
        *qualification_func.id(),
        qualification_args_json,
        qualification_prototype_context,
        "Kubeval YAML".to_owned(),
    )
    .await?;

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
        code_generation_prototype_context,
    )
    .await?;

    // TODO: it's not clear if these are the desired sockets
    // Note: This is not right; each schema needs its own socket types.
    //       Also, they should clearly inherit from the core schema? Something
    //       for later.
    let input_socket = Socket::new(
        ctx,
        "input",
        &SocketEdgeKind::Configures,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, input_socket.id()).await?;

    let output_socket = Socket::new(
        ctx,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, output_socket.id()).await?;

    let includes_socket = Socket::new(
        ctx,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await?;
    variant.add_socket(ctx, includes_socket.id()).await?;

    // TODO: abstract this boilerplate away
    let mut ui_menu = UiMenu::new(ctx, &(*schema.kind()).into()).await?;
    ui_menu.set_name(ctx, Some("deployment".to_owned())).await?;

    ui_menu.set_category(ctx, Some("kubernetes")).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let application_name = "application".to_string();
    let application_schema_results = Schema::find_by_attr(ctx, "name", &application_name).await?;
    let application_schema = application_schema_results
        .first()
        .ok_or(SchemaError::NotFoundByName(application_name))?;
    ui_menu
        .add_root_schematic(ctx, application_schema.id())
        .await?;

    Ok(())
}
