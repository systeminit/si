use crate::schema::{SchemaResult, SchemaVariant, UiMenu};
use crate::{
    HistoryActor, Schema, SchemaError, SchemaKind, SchematicKind, StandardModel, Tenancy,
    Visibility,
};
use si_data::{NatsTxn, PgTxn};

pub async fn migrate(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    application(txn, nats).await?;
    service(txn, nats).await?;

    kubernetes_service(txn, nats).await?;
    Ok(())
}

async fn application(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    let (tenancy, visibility, history_actor) = default_migration_config();
    let mut schema = Schema::new(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        "application",
        &SchemaKind::Concept,
    )
    .await?;
    schema
        .set_ui_hidden(txn, nats, &visibility, &history_actor, true)
        .await?;

    let variant =
        SchemaVariant::new(txn, nats, &tenancy, &visibility, &history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await?;

    schema
        .set_default_schema_variant_id(txn, nats, &visibility, &history_actor, Some(*variant.id()))
        .await?;

    Ok(())
}

async fn kubernetes_service(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    let (tenancy, visibility, history_actor) = default_migration_config();
    let mut schema = Schema::new(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        "kubernetes_service",
        &SchemaKind::Implementation,
    )
    .await?;

    let variant =
        SchemaVariant::new(txn, nats, &tenancy, &visibility, &history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await?;

    schema
        .set_default_schema_variant_id(txn, nats, &visibility, &history_actor, Some(*variant.id()))
        .await?;

    Ok(())
}

async fn service(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    let (tenancy, visibility, history_actor) = default_migration_config();
    let mut schema = Schema::new(
        txn,
        nats,
        &tenancy,
        &visibility,
        &history_actor,
        "service",
        &SchemaKind::Concept,
    )
    .await?;

    let mut ui_menu = UiMenu::new(txn, nats, &tenancy, &visibility, &history_actor).await?;
    ui_menu
        .set_name(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(schema.name().to_string()),
        )
        .await?;
    ui_menu
        .set_category(txn, nats, &visibility, &history_actor, Some("application"))
        .await?;
    ui_menu
        .set_schematic_kind(
            txn,
            nats,
            &visibility,
            &history_actor,
            SchematicKind::Deployment,
        )
        .await?;
    ui_menu
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await?;

    let application_schema_results = Schema::find_by_attr(
        txn,
        &tenancy,
        &visibility,
        "name",
        &String::from("application"),
    )
    .await?;
    let application_schema = application_schema_results
        .first()
        .ok_or_else(|| SchemaError::NotFoundByName("application".to_string()))?;
    ui_menu
        .add_root_schematic(
            txn,
            nats,
            &visibility,
            &history_actor,
            application_schema.id(),
        )
        .await?;

    let variant =
        SchemaVariant::new(txn, nats, &tenancy, &visibility, &history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await?;

    schema
        .set_default_schema_variant_id(txn, nats, &visibility, &history_actor, Some(*variant.id()))
        .await?;

    Ok(())
}

fn default_migration_config() -> (Tenancy, Visibility, HistoryActor) {
    (
        Tenancy::new_universal(),
        Visibility::new_head(false),
        HistoryActor::SystemInit,
    )
}
