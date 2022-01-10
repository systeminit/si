use crate::schema::defaults;
use crate::schema::{SchemaResult, UiMenu};
use crate::{
    HistoryActor, Schema, SchemaError, SchemaKind, SchematicKind, StandardModel, Tenancy,
    Visibility,
};
use si_data::{NatsTxn, PgTxn};

// NOTE(nick): the next time this becomes more complex, we should consider using a struct that
// contains tenancy, visibility, and history actor. Moreover, it should probably contain mostly
// everything but the migrate public function as methods. This will likely come to fruition as
// a natural chain of events.
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
    let mut schema = match defaults::create_schema(
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

    defaults::create_and_set_default_schema_variant(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &mut schema,
    )
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
    let mut schema = match defaults::create_schema(
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

    defaults::create_and_set_default_schema_variant(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &mut schema,
    )
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
    let mut schema = match defaults::create_schema(
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

    defaults::create_and_set_default_schema_variant(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &mut schema,
    )
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
    let mut schema = match defaults::create_schema(
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

    defaults::create_and_set_default_schema_variant(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &mut schema,
    )
    .await?;

    Ok(())
}
