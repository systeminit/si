use std::collections::HashMap;

use crate::func::backend::js_attribute::FuncBackendJsAttributeArgs;
use crate::func::backend::js_qualification::FuncBackendJsQualificationArgs;
use crate::func::backend::js_resource::FuncBackendJsResourceSyncArgs;
use crate::func::backend::validation::FuncBackendValidateStringValueArgs;
use crate::qualification_prototype::QualificationPrototypeContext;
use crate::resource_prototype::ResourcePrototypeContext;
use crate::schema::{SchemaResult, SchemaVariant, UiMenu};
use crate::socket::{Socket, SocketArity, SocketEdgeKind};
use crate::{
    component::ComponentKind, edit_field::widget::*, func::binding::FuncBinding,
    validation_prototype::ValidationPrototypeContext, Func, FuncBackendKind,
    FuncBackendResponseType, HistoryActor, Prop, PropId, PropKind, QualificationPrototype,
    ResourcePrototype, Schema, SchemaError, SchemaKind, SchemaVariantId, SchematicKind,
    StandardModel, ValidationPrototype, Visibility, WriteTenancy,
};
use crate::{
    AttributeContext, AttributePrototype, AttributePrototypeError, AttributeReadContext,
    AttributeValue, AttributeValueError, SchemaId,
};
use si_data::{NatsTxn, PgTxn};
use telemetry::prelude::*;
use veritech::EncryptionKey;

mod kubernetes_deployment;
mod kubernetes_metadata;
mod kubernetes_selector;
mod kubernetes_spec;
mod kubernetes_template;

use self::kubernetes_deployment::kubernetes_deployment;

pub async fn migrate(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let (tenancy, visibility, history_actor) = (
        WriteTenancy::new_universal(),
        Visibility::new_head(false),
        HistoryActor::SystemInit,
    );

    system(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encryption_key,
    )
    .await?;
    application(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encryption_key,
    )
    .await?;
    service(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encryption_key,
    )
    .await?;
    kubernetes_service(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encryption_key,
    )
    .await?;
    kubernetes_deployment(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encryption_key,
    )
    .await?;
    docker_image(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encryption_key,
    )
    .await?;
    docker_hub_credential(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech,
        encryption_key,
    )
    .await?;

    Ok(())
}

async fn system(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let name = "system".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        &name,
        &SchemaKind::Concept,
    )
    .await?
    {
        Some(schema) => schema,
        None => return Ok(()),
    };

    schema
        .set_ui_hidden(txn, nats, visibility, history_actor, true)
        .await?;

    let (variant, _) = SchemaVariant::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *schema.id(),
        "v0",
        veritech,
        encryption_key,
    )
    .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

    let output_socket = Socket::new(
        txn,
        nats,
        write_tenancy,
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

    Ok(())
}

async fn application(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let name = "application".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        &name,
        &SchemaKind::Concept,
    )
    .await?
    {
        Some(schema) => schema,
        None => return Ok(()),
    };

    schema
        .set_ui_hidden(txn, nats, visibility, history_actor, true)
        .await?;

    let (variant, _) = SchemaVariant::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *schema.id(),
        "v0",
        veritech,
        encryption_key,
    )
    .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

    let input_socket = Socket::new(
        txn,
        nats,
        write_tenancy,
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
        write_tenancy,
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
        write_tenancy,
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

    Ok(())
}

async fn service(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let name = "service".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        &name,
        &SchemaKind::Concept,
    )
    .await?
    {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let mut ui_menu = UiMenu::new(txn, nats, write_tenancy, visibility, history_actor).await?;
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

    let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;
    let application_schema_results = Schema::find_by_attr(
        txn,
        &(&read_tenancy).into(),
        visibility,
        "name",
        &application_name,
    )
    .await?;
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

    let (variant, _) = SchemaVariant::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *schema.id(),
        "v0",
        veritech,
        encryption_key,
    )
    .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

    let input_socket = Socket::new(
        txn,
        nats,
        write_tenancy,
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
        write_tenancy,
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
        write_tenancy,
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

    Ok(())
}

async fn kubernetes_service(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let name = "kubernetes_service".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        &name,
        &SchemaKind::Implementation,
    )
    .await?
    {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (variant, _) = SchemaVariant::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *schema.id(),
        "v0",
        veritech,
        encryption_key,
    )
    .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

    let input_socket = Socket::new(
        txn,
        nats,
        write_tenancy,
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
        write_tenancy,
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
        write_tenancy,
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

    Ok(())
}

async fn docker_hub_credential(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let name = "docker_hub_credential".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        write_tenancy,
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
    schema
        .set_component_kind(
            txn,
            nats,
            visibility,
            history_actor,
            ComponentKind::Credential,
        )
        .await?;

    let (variant, root_prop) = SchemaVariant::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *schema.id(),
        "v0",
        veritech.clone(),
        encryption_key,
    )
    .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

    let mut secret_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "secret",
        PropKind::Integer,
    )
    .await?;
    secret_prop
        .set_parent_prop(
            txn,
            nats,
            visibility,
            history_actor,
            root_prop.domain_prop_id,
        )
        .await?;
    secret_prop
        .set_widget_kind(
            txn,
            nats,
            visibility,
            history_actor,
            WidgetKind::SecretSelect,
        )
        .await?;

    // Qualification Prototype
    let qual_func_name = "si:qualificationDockerHubLogin".to_string();
    let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;
    let qual_func = Func::find_by_attr(
        txn,
        &(&read_tenancy).into(),
        visibility,
        "name",
        &qual_func_name,
    )
    .await?
    .pop()
    .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
    let qual_args = FuncBackendJsQualificationArgs::default();
    let qual_args_json = serde_json::to_value(&qual_args)?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*variant.id());

    let mut prototype = QualificationPrototype::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *qual_func.id(),
        qual_args_json,
        qual_prototype_context,
        "docker hub login credentials must work",
    )
    .await?;
    prototype
        .set_link(
            txn,
            nats,
            visibility,
            history_actor,
            "http://hub.docker.com".into(),
        )
        .await?;

    // Note: This is not right; each schema needs its own socket types.
    //       Also, they should clearly inherit from the core schema? Something
    //       for later.
    let input_socket = Socket::new(
        txn,
        nats,
        write_tenancy,
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
        write_tenancy,
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
        write_tenancy,
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

    let mut ui_menu = UiMenu::new(txn, nats, write_tenancy, visibility, history_actor).await?;
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
            SchematicKind::Deployment, // Note: This isn't right, but we are not fixing the SchematicKind::Component UI in this PR
        )
        .await?;
    ui_menu
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;

    let application_schema_results = Schema::find_by_attr(
        txn,
        &(&read_tenancy).into(),
        visibility,
        "name",
        &application_name,
    )
    .await?;
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

async fn docker_image(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<()> {
    let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;
    let name = "docker_image".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        &name,
        &SchemaKind::Concept,
    )
    .await?
    {
        Some(schema) => schema,
        None => return Ok(()),
    };

    let (variant, root_prop) = SchemaVariant::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *schema.id(),
        "v0",
        veritech.clone(),
        encryption_key,
    )
    .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*variant.id());
    let root_prop_attribute_value = AttributeValue::find_for_context(
        txn,
        &read_tenancy,
        visibility,
        AttributeReadContext {
            prop_id: Some(root_prop.domain_prop_id),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*variant.id()),
            ..AttributeReadContext::default()
        },
    )
    .await?
    .pop()
    .ok_or(AttributeValueError::Missing)?;

    let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;
    let mut ui_menu = UiMenu::new(txn, nats, write_tenancy, visibility, history_actor).await?;
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

    let application_schema_results = Schema::find_by_attr(
        txn,
        &(&read_tenancy).into(),
        visibility,
        "name",
        &application_name,
    )
    .await?;
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

    let image_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "image",
        PropKind::String,
    )
    .await?;
    image_prop
        .set_parent_prop(
            txn,
            nats,
            visibility,
            history_actor,
            root_prop.domain_prop_id,
        )
        .await?;

    // TODO: required, validate regex: "\\d+\\/(tcp|udp)", message: "invalid exposed port entry; must be [numeric]/(tcp|udp)",
    let exposed_ports_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "ExposedPorts",
        PropKind::Array, // Note: we should have a way to specify that this is an array of Integer
    )
    .await?;
    exposed_ports_prop
        .set_parent_prop(
            txn,
            nats,
            visibility,
            history_actor,
            root_prop.domain_prop_id,
        )
        .await?;

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs =
        Func::find_by_attr(txn, &(&read_tenancy).into(), visibility, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let mut validation_prototype_ctx = ValidationPrototypeContext::default();
    validation_prototype_ctx.set_prop_id(*image_prop.id());
    validation_prototype_ctx.set_schema_id(*schema.id());
    validation_prototype_ctx.set_schema_variant_id(*variant.id());
    ValidationPrototype::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "gambiarra".to_owned(),
        ))?,
        validation_prototype_ctx.clone(),
    )
    .await?;

    ValidationPrototype::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *func.id(),
        serde_json::to_value(&FuncBackendValidateStringValueArgs::new(
            None,
            "Tupi or not Tupi, that is the question".to_owned(), // https://en.wikipedia.org/wiki/Manifesto_Antrop%C3%B3fago
        ))?,
        validation_prototype_ctx,
    )
    .await?;

    let number_of_parents_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "Number of Parents",
        PropKind::String, // Should be integer, but the js integer backend isn't 100% there yet
    )
    .await?;
    number_of_parents_prop
        .set_parent_prop(
            txn,
            nats,
            visibility,
            history_actor,
            root_prop.domain_prop_id,
        )
        .await?;

    // TODO: we don't have a component to have their props, but we can manually rebuild the props from what we created in this schema variant
    // This means if someone updates this function the properties will be invalid
    let mut properties = HashMap::new();
    properties.insert("image".to_owned(), serde_json::json!(""));
    properties.insert("Number of Parents".to_owned(), serde_json::json!("0"));

    let func_name = "si:numberOfParents".to_string();
    let mut funcs =
        Func::find_by_attr(txn, &(&read_tenancy).into(), visibility, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let (func_binding, _) = FuncBinding::find_or_create(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        serde_json::to_value(FuncBackendJsAttributeArgs {
            component: veritech::ResolverFunctionComponent {
                data: veritech::ComponentView {
                    properties: serde_json::to_value(properties)?,
                    kind: veritech::ComponentKind::Standard,
                    system: None,
                },
                parents: vec![],
            },
        })?,
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    func_binding
        .execute(txn, nats, veritech.clone(), encryption_key)
        .await?;

    let number_of_parents_prop_context = attribute_context_builder
        .clone()
        .set_prop_id(*number_of_parents_prop.id())
        .to_context()?;
    let attribute_prototype = AttributePrototype::list_for_context(
        txn,
        &read_tenancy,
        visibility,
        number_of_parents_prop_context,
    )
    .await?
    .pop()
    .ok_or(AttributePrototypeError::Missing)?;
    AttributePrototype::update_for_context(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *attribute_prototype.id(),
        number_of_parents_prop_context,
        *func.id(),
        *func_binding.id(),
        Some(*root_prop_attribute_value.id()),
        Some(
            *AttributeValue::find_for_context(
                txn,
                &read_tenancy,
                visibility,
                number_of_parents_prop_context.into(),
            )
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        ),
    )
    .await?;

    // Note: This is not right; each schema needs its own socket types.
    //       Also, they should clearly inherit from the core schema? Something
    //       for later.
    let input_socket = Socket::new(
        txn,
        nats,
        write_tenancy,
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
        write_tenancy,
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
        write_tenancy,
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

    // Qualification Prototype
    let qual_func_name = "si:qualificationDockerImageNameInspect".to_string();
    let mut qual_funcs = Func::find_by_attr(
        txn,
        &(&read_tenancy).into(),
        visibility,
        "name",
        &qual_func_name,
    )
    .await?;
    let qual_func = qual_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
    let qual_args = FuncBackendJsQualificationArgs::default();
    let qual_args_json = serde_json::to_value(&qual_args)?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*variant.id());

    let mut prototype = QualificationPrototype::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *qual_func.id(),
        qual_args_json,
        qual_prototype_context,
        "docker image name must match the component name",
    )
    .await?;
    prototype
        .set_link(
            txn,
            nats,
            visibility,
            history_actor,
            "http://docker.com".into(),
        )
        .await?;

    // Resource Prototype
    let resource_sync_func_name = "si:resourceSyncHammer".to_string();
    let mut resource_sync_funcs = Func::find_by_attr(
        txn,
        &(&read_tenancy).into(),
        visibility,
        "name",
        &resource_sync_func_name,
    )
    .await?;
    let resource_sync_func = resource_sync_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(resource_sync_func_name))?;
    let resource_sync_args = FuncBackendJsResourceSyncArgs::default();
    let resource_sync_args_json = serde_json::to_value(&resource_sync_args)?;
    let mut resource_sync_prototype_context = ResourcePrototypeContext::new();
    resource_sync_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = ResourcePrototype::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        *resource_sync_func.id(),
        resource_sync_args_json,
        resource_sync_prototype_context,
    )
    .await?;

    Ok(())
}

async fn create_schema(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    schema_name: &str,
    schema_kind: &SchemaKind,
) -> SchemaResult<Option<Schema>> {
    let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;
    // TODO(nick): there's one issue here. If the schema kind has changed, then this check will be
    // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
    // This should be fine since this code should likely only last as long as default schemas need to
    // be created... which is hopefully not long.... hopefully...
    let default_schema_exists = !Schema::find_by_attr(
        txn,
        &(&read_tenancy).into(),
        visibility,
        "name",
        &schema_name.to_string(),
    )
    .await?
    .is_empty();

    // TODO(nick): this should probably return an "AlreadyExists" error instead of "None", but
    // since the calling function would have to deal with the result similarly, this should suffice
    // for now.
    match default_schema_exists {
        true => Ok(None),
        false => {
            let schema = Schema::new(
                txn,
                nats,
                write_tenancy,
                visibility,
                history_actor,
                schema_name,
                schema_kind,
                &ComponentKind::Standard,
            )
            .await?;
            Ok(Some(schema))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    prop_name: &str,
    prop_kind: PropKind,
    parent_prop_id: Option<PropId>,
) -> SchemaResult<Prop> {
    let prop = Prop::new(
        txn,
        nats,
        veritech,
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        prop_name,
        prop_kind,
    )
    .await?;
    if let Some(parent_prop_id) = parent_prop_id {
        prop.set_parent_prop(txn, nats, visibility, history_actor, parent_prop_id)
            .await?;
    }
    Ok(prop)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_string_prop_with_default(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    prop_name: &str,
    default_string: String,
    parent_prop_id: Option<PropId>,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<Prop> {
    let prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        prop_name,
        PropKind::String,
        parent_prop_id,
    )
    .await?;

    let mut func = Func::new(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        &format!("si:setDefaultToProp{}", prop.id()),
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");
    func.set_handler(txn, nats, visibility, history_actor, Some("defaultValue"))
        .await?;
    func.set_code_base64(
        txn,
        nats,
        visibility,
        history_actor,
        Some(base64::encode(&format!(
            "function defaultValue(component) {{ return \"{default_string}\"; }}"
        ))),
    )
    .await?;

    let (func_binding, created) = FuncBinding::find_or_create(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
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
        *func.backend_kind(),
    )
    .await?;

    if created {
        func_binding
            .execute(txn, nats, veritech, encryption_key)
            .await?;
    }

    // TODO: Set up AttribuePrototype & AttributeValue appropriately

    Ok(prop)
}

pub struct RootProp {
    pub si_prop_id: PropId,
    pub domain_prop_id: PropId,
}

/// Returns si-specific prop id and domain-specific prop id, respectfully
#[instrument(skip_all)]
#[allow(clippy::too_many_arguments)]
pub async fn create_root_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    schema_id: SchemaId,
    variant_id: SchemaVariantId,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<RootProp> {
    let mut variant_context_builder = AttributeContext::builder();
    variant_context_builder
        .set_schema_id(schema_id)
        .set_schema_variant_id(variant_id);

    let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;
    let func_name = "si:setPropObject".to_string();
    let mut funcs =
        Func::find_by_attr(txn, &(&read_tenancy).into(), visibility, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;

    let (func_binding, created) = FuncBinding::find_or_create(
        txn,
        nats,
        write_tenancy,
        visibility,
        history_actor,
        // Shortcut creating a FuncBackendPropObjectArgs
        serde_json::json!({ "value": {} }),
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    if created {
        func_binding
            .execute(txn, nats, veritech.clone(), encryption_key)
            .await?;
    }

    let root_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "root",
        PropKind::Object,
    )
    .await?;
    root_prop
        .add_schema_variant(txn, nats, visibility, history_actor, &variant_id)
        .await?;

    let root_context = variant_context_builder
        .clone()
        .set_prop_id(*root_prop.id())
        .to_context()?;
    let (_, root_value_id) = AttributeValue::update_for_context(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        *AttributeValue::find_for_context(txn, &read_tenancy, visibility, root_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        None,
        root_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await?;

    let si_specific_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "si",
        PropKind::Object,
    )
    .await?;
    si_specific_prop
        .set_parent_prop(txn, nats, visibility, history_actor, *root_prop.id())
        .await?;

    let si_context = variant_context_builder
        .clone()
        .set_prop_id(*si_specific_prop.id())
        .to_context()?;
    let (_, si_value_id) = AttributeValue::update_for_context(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        *AttributeValue::find_for_context(txn, &read_tenancy, visibility, si_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        Some(root_value_id),
        si_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await?;

    let si_name_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "name",
        PropKind::String,
    )
    .await?;
    si_name_prop
        .set_parent_prop(txn, nats, visibility, history_actor, *si_specific_prop.id())
        .await?;

    let si_name_context = variant_context_builder
        .clone()
        .set_prop_id(*si_name_prop.id())
        .to_context()?;
    AttributeValue::update_for_context(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        *AttributeValue::find_for_context(txn, &read_tenancy, visibility, si_name_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        Some(si_value_id),
        si_name_context,
        None,
        None,
    )
    .await?;

    let domain_specific_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "domain",
        PropKind::Object,
    )
    .await?;
    domain_specific_prop
        .set_parent_prop(txn, nats, visibility, history_actor, *root_prop.id())
        .await?;

    let domain_context = variant_context_builder
        .clone()
        .set_prop_id(*domain_specific_prop.id())
        .to_context()?;
    AttributeValue::update_for_context(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        *AttributeValue::find_for_context(txn, &read_tenancy, visibility, domain_context.into())
            .await?
            .pop()
            .ok_or(AttributeValueError::Missing)?
            .id(),
        Some(root_value_id),
        domain_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await?;

    Ok(RootProp {
        si_prop_id: *si_specific_prop.id(),
        domain_prop_id: *domain_specific_prop.id(),
    })
}
