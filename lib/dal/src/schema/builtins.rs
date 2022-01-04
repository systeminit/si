use crate::schema::{SchemaResult, SchemaVariant};
use crate::{HistoryActor, Schema, SchemaKind, StandardModel, Tenancy, Visibility};
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
