use crate::schema::{SchemaResult, SchemaVariant};
use crate::{HistoryActor, Schema, SchemaKind, StandardModel, Tenancy, Visibility};
use si_data::{NatsTxn, PgTxn};

pub async fn migrate(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    application(&txn, &nats).await?;
    Ok(())
}

async fn application(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = Schema::new(
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
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await?;
    Ok(())
}
