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
    CodeGenerationPrototype, Func, HistoryActor, PropKind, QualificationPrototype, Schema,
    SchemaError, SchemaKind, SchematicKind, StandardModel, Tenancy, Visibility,
};
use si_data::{NatsTxn, PgTxn};
use veritech::EncryptionKey;

pub async fn kubernetes_deployment(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let name = "kubernetes_deployment".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &name,
        &SchemaKind::Concrete,
    )
    .await?
    {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (variant, root_prop) = SchemaVariant::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        "v0",
        veritech.clone(),
        encryption_key,
    )
    .await?;
    variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

    {
        // TODO: add validation (si-registry ensures the value is unchanged)
        let _api_version_prop = create_string_prop_with_default(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            variant.id(),
            "apiVersion",
            "apps/v1".to_owned(),
            Some(root_prop.domain_prop_id),
            veritech.clone(),
            encryption_key,
        )
        .await?;
    }
    {
        // TODO: add validation (si-registry ensures the value is unchanged)
        let _kind_prop = create_string_prop_with_default(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            variant.id(),
            "kind",
            "Deployment".to_owned(),
            Some(root_prop.domain_prop_id),
            veritech.clone(),
            encryption_key,
        )
        .await?;
    }

    {
        let _metadata_prop = create_metadata_prop(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            variant.id(),
            true, // is name required, note: bool is not ideal here tho
            Some(root_prop.domain_prop_id),
            veritech.clone(),
            encryption_key,
        )
        .await?;
    }

    {
        let spec_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            encryption_key,
            tenancy,
            visibility,
            history_actor,
            variant.id(),
            "spec",
            PropKind::Object,
            Some(root_prop.domain_prop_id),
        )
        .await?;

        {
            let _replicas_prop = create_prop(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                tenancy,
                visibility,
                history_actor,
                variant.id(),
                "replicas",
                PropKind::Integer,
                Some(*spec_prop.id()),
            )
            .await?;
        }

        {
            let _selector_prop = create_selector_prop(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                tenancy,
                visibility,
                history_actor,
                variant.id(),
                Some(*spec_prop.id()),
            )
            .await?;
        }

        {
            let _template_prop = create_template_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant.id(),
                Some(*spec_prop.id()),
                veritech.clone(),
                encryption_key,
            )
            .await?;
        }
    }

    // Qualification Prototype
    let qualification_func_name = "si:qualificationYamlKubeval".to_owned();
    let mut qualification_funcs =
        Func::find_by_attr(txn, tenancy, visibility, "name", &qualification_func_name).await?;
    let qualification_func = qualification_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qualification_func_name))?;
    let qualification_args = FuncBackendJsQualificationArgs::default();
    let qualification_args_json = serde_json::to_value(&qualification_args)?;
    let mut qualification_prototype_context = QualificationPrototypeContext::new();
    qualification_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = QualificationPrototype::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        *qualification_func.id(),
        qualification_args_json,
        qualification_prototype_context,
        "Kubeval YAML".to_owned(),
    )
    .await?;

    // Code Generation Prototype
    let code_generation_func_name = "si:generateYAML".to_owned();
    let mut code_generation_funcs =
        Func::find_by_attr(txn, tenancy, visibility, "name", &code_generation_func_name).await?;
    let code_generation_func = code_generation_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;
    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = CodeGenerationPrototype::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
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
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        "input",
        &SocketEdgeKind::Configures,
        &SocketArity::Many,
    )
    .await?;
    variant
        .add_socket(txn, nats, visibility, history_actor, input_socket.id())
        .await?;

    let output_socket = Socket::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
    )
    .await?;
    variant
        .add_socket(txn, nats, visibility, history_actor, output_socket.id())
        .await?;

    let includes_socket = Socket::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
    )
    .await?;
    variant
        .add_socket(txn, nats, visibility, history_actor, includes_socket.id())
        .await?;

    // TODO: abstract this boilerplate away
    let mut ui_menu = UiMenu::new(txn, nats, tenancy, visibility, history_actor).await?;
    ui_menu
        .set_name(
            txn,
            nats,
            visibility,
            history_actor,
            Some(schema.name().to_string()),
        )
        .await?;

    let application_name = "application".to_string();
    ui_menu
        .set_category(
            txn,
            nats,
            visibility,
            history_actor,
            Some(application_name.clone()),
        )
        .await?;
    ui_menu
        .set_schematic_kind(
            txn,
            nats,
            visibility,
            history_actor,
            SchematicKind::Deployment,
        )
        .await?;
    ui_menu
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;

    let application_schema_results =
        Schema::find_by_attr(txn, tenancy, visibility, "name", &application_name).await?;
    let application_schema = application_schema_results
        .first()
        .ok_or(SchemaError::NotFoundByName(application_name))?;
    ui_menu
        .add_root_schematic(
            txn,
            nats,
            visibility,
            history_actor,
            application_schema.id(),
        )
        .await?;

    Ok(())
}
