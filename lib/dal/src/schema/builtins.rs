use crate::func::backend::validation::FuncBackendValidateStringValueArgs;
use crate::schema::{SchemaResult, SchemaVariant, UiMenu};
use crate::socket::{Socket, SocketArity, SocketEdgeKind};
use crate::{
    validation_prototype::ValidationPrototypeContext, Func, HistoryActor, Prop, PropKind, Schema,
    SchemaError, SchemaKind, SchematicKind, StandardModel, Tenancy, ValidationPrototype,
    Visibility,
};
use si_data::{NatsTxn, PgTxn};

pub async fn migrate(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    let (tenancy, visibility, history_actor) = (
        Tenancy::new_universal(),
        Visibility::new_head(false),
        HistoryActor::SystemInit,
    );

    application(txn, nats, &tenancy, &visibility, &history_actor).await?;
    service(txn, nats, &tenancy, &visibility, &history_actor).await?;
    kubernetes_service(txn, nats, &tenancy, &visibility, &history_actor).await?;
    docker_image(txn, nats, &tenancy, &visibility, &history_actor).await?;

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

    Ok(())
}

async fn docker_image(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
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
        validation_prototype_ctx,
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
