use std::collections::HashMap;

use crate::func::backend::js_qualification::FuncBackendJsQualificationArgs;
use crate::func::backend::js_resource::FuncBackendJsResourceSyncArgs;
use crate::func::backend::js_string::FuncBackendJsStringArgs;
use crate::func::backend::string::FuncBackendStringArgs;
use crate::func::backend::validation::FuncBackendValidateStringValueArgs;
use crate::qualification_prototype::QualificationPrototypeContext;
use crate::resource_prototype::ResourcePrototypeContext;
use crate::schema::{SchemaResult, SchemaVariant, UiMenu};
use crate::socket::{Socket, SocketArity, SocketEdgeKind};
use crate::{
    attribute_resolver::AttributeResolverContext, func::binding::FuncBinding,
    validation_prototype::ValidationPrototypeContext, AttributeResolver, Func, HistoryActor, Prop,
    PropId, PropKind, QualificationPrototype, ResourcePrototype, Schema, SchemaError, SchemaKind,
    SchemaVariantId, SchematicKind, StandardModel, Tenancy, ValidationPrototype, Visibility,
};
use si_data::{NatsTxn, PgTxn};

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
) -> SchemaResult<()> {
    let (tenancy, visibility, history_actor) = (
        Tenancy::new_universal(),
        Visibility::new_head(false),
        HistoryActor::SystemInit,
    );

    system(txn, nats, &tenancy, &visibility, &history_actor).await?;
    application(txn, nats, &tenancy, &visibility, &history_actor).await?;
    service(txn, nats, &tenancy, &visibility, &history_actor).await?;
    kubernetes_service(txn, nats, &tenancy, &visibility, &history_actor).await?;
    kubernetes_deployment(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
    )
    .await?;
    docker_image(txn, nats, &tenancy, &visibility, &history_actor, veritech).await?;

    Ok(())
}

async fn system(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> SchemaResult<()> {
    let name = "system".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        tenancy,
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

    let variant = SchemaVariant::new(txn, nats, tenancy, visibility, history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
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

    Ok(())
}

async fn application(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> SchemaResult<()> {
    let name = "application".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        tenancy,
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

    let variant = SchemaVariant::new(txn, nats, tenancy, visibility, history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

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

    Ok(())
}

async fn service(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> SchemaResult<()> {
    let name = "service".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        tenancy,
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

    let variant = SchemaVariant::new(txn, nats, tenancy, visibility, history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

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

    Ok(())
}

async fn kubernetes_service(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> SchemaResult<()> {
    let name = "kubernetes_service".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        tenancy,
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

    let variant = SchemaVariant::new(txn, nats, tenancy, visibility, history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

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

    Ok(())
}

async fn docker_image(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
) -> SchemaResult<()> {
    let name = "docker_image".to_string();
    let mut schema = match create_schema(
        txn,
        nats,
        tenancy,
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

    let variant = SchemaVariant::new(txn, nats, tenancy, visibility, history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await?;

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

    let image_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        tenancy,
        visibility,
        history_actor,
        "image",
        PropKind::String,
    )
    .await?;
    image_prop
        .add_schema_variant(txn, nats, visibility, history_actor, variant.id())
        .await?;

    // TODO: required, validate regex: "\\d+\\/(tcp|udp)", message: "invalid exposed port entry; must be [numeric]/(tcp|udp)",
    let exposed_ports_prop = Prop::new(
        txn,
        nats,
        veritech.clone(),
        tenancy,
        visibility,
        history_actor,
        "ExposedPorts",
        PropKind::Array, // Note: we should have a way to specify that this is an array of Integer
    )
    .await?;
    exposed_ports_prop
        .add_schema_variant(txn, nats, visibility, history_actor, variant.id())
        .await?;

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let mut validation_prototype_ctx = ValidationPrototypeContext::default();
    validation_prototype_ctx.set_prop_id(*image_prop.id());
    validation_prototype_ctx.set_schema_id(*schema.id());
    validation_prototype_ctx.set_schema_variant_id(*variant.id());
    ValidationPrototype::new(
        txn,
        nats,
        tenancy,
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
        tenancy,
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
        tenancy,
        visibility,
        history_actor,
        "Number of Parents",
        PropKind::String, // Should be integer, but the js integer backend isn't 100% there yet
    )
    .await?;
    number_of_parents_prop
        .add_schema_variant(txn, nats, visibility, history_actor, variant.id())
        .await?;

    // TODO: we don't have a component to have their props, but we can manually rebuild the props from what we created in this schema variant
    // This means if someone updates this function the properties will be invalid
    let mut properties = HashMap::new();
    properties.insert("image".to_owned(), serde_json::json!(""));
    properties.insert("Number of Parents".to_owned(), serde_json::json!("0"));

    let func_name = "si:numberOfParents".to_string();
    let mut funcs = Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let (func_binding, _) = FuncBinding::find_or_create(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        serde_json::to_value(FuncBackendJsStringArgs {
            component: veritech::ResolverFunctionComponent {
                name: number_of_parents_prop.name().to_owned(),
                properties,
                // TODO: can there be a component connected to another during schema variant creation?
                parents: vec![],
            },
        })?,
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    func_binding.execute(txn, nats, veritech.clone()).await?;

    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*number_of_parents_prop.id());
    AttributeResolver::upsert(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await?;

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

    // Qualification Prototype
    let qual_func_name = "si:qualificationDockerImageNameInspect".to_string();
    let mut qual_funcs =
        Func::find_by_attr(txn, tenancy, visibility, "name", &qual_func_name).await?;
    let qual_func = qual_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(qual_func_name))?;
    let qual_args = FuncBackendJsQualificationArgs {
        component: veritech::QualificationCheckComponent::default(),
    };
    let qual_args_json = serde_json::to_value(&qual_args)?;
    let mut qual_prototype_context = QualificationPrototypeContext::new();
    qual_prototype_context.set_schema_variant_id(*variant.id());

    let mut prototype = QualificationPrototype::new(
        txn,
        nats,
        tenancy,
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
    let mut resource_sync_funcs =
        Func::find_by_attr(txn, tenancy, visibility, "name", &resource_sync_func_name).await?;
    let resource_sync_func = resource_sync_funcs
        .pop()
        .ok_or(SchemaError::FuncNotFound(resource_sync_func_name))?;
    let resource_sync_args = FuncBackendJsResourceSyncArgs {
        component: veritech::ResourceSyncComponent::default(),
    };
    let resource_sync_args_json = serde_json::to_value(&resource_sync_args)?;
    let mut resource_sync_prototype_context = ResourcePrototypeContext::new();
    resource_sync_prototype_context.set_schema_variant_id(*variant.id());

    let _prototype = ResourcePrototype::new(
        txn,
        nats,
        tenancy,
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
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    schema_name: &str,
    schema_kind: &SchemaKind,
) -> SchemaResult<Option<Schema>> {
    // TODO(nick): there's one issue here. If the schema kind has changed, then this check will be
    // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
    // This should be fine since this code should likely only last as long as default schemas need to
    // be created... which is hopefully not long.... hopefully...
    let default_schema_exists =
        !Schema::find_by_attr(txn, tenancy, visibility, "name", &schema_name.to_string())
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
                tenancy,
                visibility,
                history_actor,
                schema_name,
                schema_kind,
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
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    variant_id: &SchemaVariantId,
    prop_name: &str,
    prop_kind: PropKind,
    parent_prop_id: Option<PropId>,
) -> SchemaResult<Prop> {
    let prop = Prop::new(
        txn,
        nats,
        veritech,
        tenancy,
        visibility,
        history_actor,
        prop_name,
        prop_kind,
    )
    .await?;
    prop.add_schema_variant(txn, nats, visibility, history_actor, variant_id)
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
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    variant_id: &SchemaVariantId,
    prop_name: &str,
    default_string: String,
    parent_prop_id: Option<PropId>,
    veritech: veritech::Client,
) -> SchemaResult<Prop> {
    let prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        tenancy,
        visibility,
        history_actor,
        variant_id,
        prop_name,
        PropKind::String,
        parent_prop_id,
    )
    .await?;

    let func_name = "si:setString".to_string();
    let mut funcs = Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
    let func = funcs.pop().ok_or(SchemaError::MissingFunc(func_name))?;
    let (func_binding, created) = FuncBinding::find_or_create(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        serde_json::to_value(FuncBackendStringArgs {
            value: default_string,
        })?,
        *func.id(),
        *func.backend_kind(),
    )
    .await?;

    if created {
        func_binding.execute(txn, nats, veritech).await?;
    }

    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*prop.id());
    AttributeResolver::upsert(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await?;
    Ok(prop)
}
