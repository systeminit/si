use crate::schema::{SchemaResult, SchemaVariant, UiMenu};
use crate::{
    HistoryActor, Schema, SchemaError, SchemaKind, SchematicKind, StandardModel, Tenancy,
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
    Ok(())
}

async fn docker_image(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> SchemaResult<()> {
    let name = "docker_imagee".to_string();
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
